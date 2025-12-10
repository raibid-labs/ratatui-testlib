//! Integration tests for OSC 133 semantic zone testing helpers.

#[cfg(feature = "ipc")]
mod zones_integration {
    use ratatui_testlib::zones::{Osc133Marker, Osc133Parser, ZoneType};

    #[test]
    fn test_complete_command_cycle() {
        let mut parser = Osc133Parser::new();

        // Simulate a complete shell command cycle
        let data = concat!(
            "\x1b]133;A\x07",     // Fresh line
            "$ ",                  // Prompt
            "\x1b]133;B\x07",     // Command start
            "ls -la",              // Command text
            "\x1b]133;C\x07",     // Command executed
            "\ntotal 32\n",        // Output
            "drwxr-xr-x 2 user\n",
            "\x1b]133;D;0\x07",   // Command finished with exit code 0
        )
        .as_bytes();

        parser.parse(data);

        let zones = parser.zones();
        assert_eq!(zones.len(), 3, "Should have prompt, command, and output zones");

        // Check prompt zone
        assert_eq!(zones[0].zone_type, ZoneType::Prompt);

        // Check command zone
        assert_eq!(zones[1].zone_type, ZoneType::Command);

        // Check output zone
        assert_eq!(zones[2].zone_type, ZoneType::Output);
        assert_eq!(zones[2].exit_code, Some(0));
    }

    #[test]
    fn test_failed_command() {
        let mut parser = Osc133Parser::new();

        // Simulate a failed command
        let data = concat!(
            "\x1b]133;A\x07",
            "$ ",
            "\x1b]133;B\x07",
            "false",
            "\x1b]133;C\x07",
            "\n",
            "\x1b]133;D;1\x07", // Exit code 1 (failure)
        )
        .as_bytes();

        parser.parse(data);

        let zones = parser.zones();
        assert_eq!(zones.len(), 3);

        // Output zone should have exit code 1
        let output_zone = zones.iter().find(|z| z.zone_type == ZoneType::Output);
        assert!(output_zone.is_some());
        assert_eq!(output_zone.unwrap().exit_code, Some(1));
    }

    #[test]
    fn test_multiple_commands() {
        let mut parser = Osc133Parser::new();

        // Simulate multiple commands in sequence
        let data = concat!(
            // First command
            "\x1b]133;A\x07$ ",
            "\x1b]133;B\x07echo hello",
            "\x1b]133;C\x07\nhello\n",
            "\x1b]133;D;0\x07",
            // Second command
            "\x1b]133;A\x07$ ",
            "\x1b]133;B\x07pwd",
            "\x1b]133;C\x07\n/home/user\n",
            "\x1b]133;D;0\x07",
        )
        .as_bytes();

        parser.parse(data);

        let zones = parser.zones();
        // Should have 2 prompts + 2 commands + 2 outputs = 6 zones
        assert_eq!(zones.len(), 6);

        // Count zone types
        let prompts = zones.iter().filter(|z| z.zone_type == ZoneType::Prompt).count();
        let commands = zones.iter().filter(|z| z.zone_type == ZoneType::Command).count();
        let outputs = zones.iter().filter(|z| z.zone_type == ZoneType::Output).count();

        assert_eq!(prompts, 2);
        assert_eq!(commands, 2);
        assert_eq!(outputs, 2);
    }

    #[test]
    fn test_marker_detection() {
        let mut parser = Osc133Parser::new();

        parser.parse(b"\x1b]133;A\x07");
        assert_eq!(parser.markers().len(), 1);
        assert_eq!(parser.markers()[0].0, Osc133Marker::FreshLine);

        parser.clear();
        parser.parse(b"\x1b]133;B\x07");
        assert_eq!(parser.markers().len(), 1);
        assert_eq!(parser.markers()[0].0, Osc133Marker::CommandStart);

        parser.clear();
        parser.parse(b"\x1b]133;C\x07");
        assert_eq!(parser.markers().len(), 1);
        assert_eq!(parser.markers()[0].0, Osc133Marker::CommandExecuted);

        parser.clear();
        parser.parse(b"\x1b]133;D;127\x07");
        assert_eq!(parser.markers().len(), 1);
        assert_eq!(
            parser.markers()[0].0,
            Osc133Marker::CommandFinished(Some(127))
        );
    }

    #[test]
    fn test_mixed_content() {
        let mut parser = Osc133Parser::new();

        // Mix OSC 133 with regular ANSI sequences
        let data = concat!(
            "\x1b]133;A\x07",
            "\x1b[31m$ \x1b[0m", // Red prompt
            "\x1b]133;B\x07",
            "\x1b[1mls\x1b[0m", // Bold ls
            "\x1b]133;C\x07",
            "\x1b[32mfile.txt\x1b[0m\n", // Green file name
            "\x1b]133;D;0\x07",
        )
        .as_bytes();

        parser.parse(data);

        let markers = parser.markers();
        assert_eq!(markers.len(), 4);

        let zones = parser.zones();
        assert_eq!(zones.len(), 3);
    }

    #[test]
    fn test_parser_reuse() {
        let mut parser = Osc133Parser::new();

        // First parse
        parser.parse(b"\x1b]133;A\x07$ \x1b]133;B\x07");
        assert_eq!(parser.markers().len(), 2);

        // Clear and reuse
        parser.clear();
        assert_eq!(parser.markers().len(), 0);

        // Second parse
        parser.parse(b"\x1b]133;C\x07\n\x1b]133;D;0\x07");
        assert_eq!(parser.markers().len(), 2);
    }

    #[test]
    fn test_exit_code_variations() {
        let test_cases = vec![
            (b"\x1b]133;D\x07" as &[u8], None),
            (b"\x1b]133;D;0\x07", Some(0)),
            (b"\x1b]133;D;1\x07", Some(1)),
            (b"\x1b]133;D;127\x07", Some(127)),
            (b"\x1b]133;D;255\x07", Some(255)),
        ];

        for (data, expected) in test_cases {
            let mut parser = Osc133Parser::new();
            parser.parse(data);

            assert_eq!(parser.markers().len(), 1);
            match parser.markers()[0].0 {
                Osc133Marker::CommandFinished(code) => assert_eq!(code, expected),
                _ => panic!("Expected CommandFinished marker"),
            }
        }
    }

    #[test]
    fn test_incomplete_sequences() {
        let mut parser = Osc133Parser::new();

        // Only A and B markers (no C and D)
        parser.parse(b"\x1b]133;A\x07$ \x1b]133;B\x07ls");

        let markers = parser.markers();
        assert_eq!(markers.len(), 2);

        let zones = parser.zones();
        // Should have one prompt zone (A to B)
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].zone_type, ZoneType::Prompt);
    }

    #[test]
    fn test_zone_boundaries() {
        let mut parser = Osc133Parser::new();

        // Parse with known positions
        parser.parse(b"\x1b]133;A\x07$ \x1b]133;B\x07ls\x1b]133;C\x07\noutput\n\x1b]133;D;0\x07");

        let zones = parser.zones();

        // All zones should have valid boundaries
        for zone in &zones {
            assert!(zone.end_row >= zone.start_row);
            if zone.start_row == zone.end_row {
                assert!(zone.end_col >= zone.start_col);
            }
        }
    }

    #[test]
    fn test_default_constructor() {
        let parser = Osc133Parser::default();
        assert_eq!(parser.markers().len(), 0);
        assert_eq!(parser.zones().len(), 0);
    }
}

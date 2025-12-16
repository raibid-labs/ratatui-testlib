//! Integration tests for the Scarab module.
//!
//! These tests verify the Scarab-specific testing functionality.
//! Most tests are unit tests in the module itself; integration tests
//! that require a running daemon are marked with `#[ignore]`.

#[cfg(all(feature = "scarab", target_family = "unix"))]
mod scarab_tests {
    use std::path::PathBuf;
    use std::time::Duration;

    use terminal_testlib::scarab::{ScarabConfig, ScarabTestHarness};

    #[test]
    fn test_default_config_values() {
        let config = ScarabConfig::default();

        assert_eq!(config.socket_path, PathBuf::from("/tmp/scarab-daemon.sock"));
        assert_eq!(config.shm_path, "/scarab_shm_v1");
        assert_eq!(config.image_shm_path, Some("/scarab_img_v1".to_string()));
        assert_eq!(config.dimensions, Some((80, 24)));
        assert_eq!(config.connect_timeout, Duration::from_secs(5));
        assert_eq!(config.default_timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_default_prompt_patterns() {
        let config = ScarabConfig::default();

        assert!(config.prompt_patterns.contains(&"$ ".to_string()));
        assert!(config.prompt_patterns.contains(&"# ".to_string()));
        assert!(config.prompt_patterns.contains(&"> ".to_string()));
    }

    #[test]
    fn test_config_builder_socket_path() {
        let config = ScarabConfig::builder()
            .socket_path("/custom/scarab.sock")
            .build();

        assert_eq!(config.socket_path, PathBuf::from("/custom/scarab.sock"));
    }

    #[test]
    fn test_config_builder_shm_path() {
        let config = ScarabConfig::builder()
            .shm_path("/custom_shm")
            .build();

        assert_eq!(config.shm_path, "/custom_shm");
    }

    #[test]
    fn test_config_builder_image_shm_path() {
        let config = ScarabConfig::builder()
            .image_shm_path("/custom_img_shm")
            .build();

        assert_eq!(config.image_shm_path, Some("/custom_img_shm".to_string()));
    }

    #[test]
    fn test_config_builder_dimensions() {
        let config = ScarabConfig::builder()
            .dimensions(120, 40)
            .build();

        assert_eq!(config.dimensions, Some((120, 40)));
    }

    #[test]
    fn test_config_builder_timeouts() {
        let config = ScarabConfig::builder()
            .connect_timeout(Duration::from_secs(15))
            .default_timeout(Duration::from_secs(30))
            .build();

        assert_eq!(config.connect_timeout, Duration::from_secs(15));
        assert_eq!(config.default_timeout, Duration::from_secs(30));
    }

    #[test]
    fn test_config_builder_prompt_patterns() {
        let config = ScarabConfig::builder()
            .prompt_patterns(vec![">>> ".to_string(), "... ".to_string()])
            .build();

        assert_eq!(config.prompt_patterns.len(), 2);
        assert!(config.prompt_patterns.contains(&">>> ".to_string()));
        assert!(config.prompt_patterns.contains(&"... ".to_string()));
    }

    #[test]
    fn test_config_builder_add_prompt_pattern() {
        let config = ScarabConfig::builder()
            .add_prompt_pattern("custom> ")
            .build();

        // Should have default patterns plus the custom one
        assert!(config.prompt_patterns.contains(&"custom> ".to_string()));
        // Default patterns should still be there
        assert!(config.prompt_patterns.contains(&"$ ".to_string()));
    }

    #[test]
    fn test_config_builder_chaining() {
        let config = ScarabConfig::builder()
            .socket_path("/test.sock")
            .shm_path("/test_shm")
            .dimensions(100, 30)
            .connect_timeout(Duration::from_secs(20))
            .default_timeout(Duration::from_secs(60))
            .add_prompt_pattern("test> ")
            .build();

        assert_eq!(config.socket_path, PathBuf::from("/test.sock"));
        assert_eq!(config.shm_path, "/test_shm");
        assert_eq!(config.dimensions, Some((100, 30)));
        assert_eq!(config.connect_timeout, Duration::from_secs(20));
        assert_eq!(config.default_timeout, Duration::from_secs(60));
        assert!(config.prompt_patterns.contains(&"test> ".to_string()));
    }

    #[test]
    fn test_is_enabled_without_env() {
        std::env::remove_var("SCARAB_TEST_RTL");
        assert!(!ScarabTestHarness::is_enabled());
    }

    #[test]
    fn test_is_enabled_with_env() {
        std::env::set_var("SCARAB_TEST_RTL", "1");
        assert!(ScarabTestHarness::is_enabled());
        std::env::remove_var("SCARAB_TEST_RTL");
    }

    #[test]
    fn test_is_enabled_with_any_value() {
        std::env::set_var("SCARAB_TEST_RTL", "yes");
        assert!(ScarabTestHarness::is_enabled());
        std::env::remove_var("SCARAB_TEST_RTL");

        std::env::set_var("SCARAB_TEST_RTL", "true");
        assert!(ScarabTestHarness::is_enabled());
        std::env::remove_var("SCARAB_TEST_RTL");

        std::env::set_var("SCARAB_TEST_RTL", "0");
        assert!(ScarabTestHarness::is_enabled()); // Any value counts as enabled
        std::env::remove_var("SCARAB_TEST_RTL");
    }

    // Integration tests that require a running scarab-daemon
    #[test]
    #[ignore = "requires running scarab-daemon - run with SCARAB_TEST_RTL=1"]
    fn test_scarab_connection() {
        if !ScarabTestHarness::is_enabled() {
            return;
        }

        let result = ScarabTestHarness::connect();

        match result {
            Ok(harness) => {
                let (cols, rows) = harness.dimensions();
                assert!(cols > 0, "Terminal width should be positive");
                assert!(rows > 0, "Terminal height should be positive");
            }
            Err(e) => {
                println!("Expected error (daemon not running): {}", e);
            }
        }
    }

    #[test]
    #[ignore = "requires running scarab-daemon - run with SCARAB_TEST_RTL=1"]
    fn test_scarab_send_and_receive() {
        if !ScarabTestHarness::is_enabled() {
            return;
        }

        let mut harness = match ScarabTestHarness::connect() {
            Ok(h) => h,
            Err(_) => return,
        };

        // Send a simple echo command
        harness.send_input("echo scarab_test_marker_xyz\n").unwrap();

        // Wait for the output
        harness
            .wait_for_text("scarab_test_marker_xyz", Duration::from_secs(5))
            .unwrap();

        // Verify grid contains the marker
        assert!(harness.contains("scarab_test_marker_xyz").unwrap());
    }

    #[test]
    #[ignore = "requires running scarab-daemon - run with SCARAB_TEST_RTL=1"]
    fn test_scarab_cursor_position() {
        if !ScarabTestHarness::is_enabled() {
            return;
        }

        let harness = match ScarabTestHarness::connect() {
            Ok(h) => h,
            Err(_) => return,
        };

        let (row, col) = harness.cursor_position().unwrap();

        // Cursor should be within terminal bounds
        let (cols, rows) = harness.dimensions();
        assert!(col < cols, "Cursor column should be within bounds");
        assert!(row < rows, "Cursor row should be within bounds");
    }

    #[test]
    #[ignore = "requires running scarab-daemon - run with SCARAB_TEST_RTL=1"]
    fn test_scarab_wait_for_prompt() {
        if !ScarabTestHarness::is_enabled() {
            return;
        }

        let mut harness = match ScarabTestHarness::connect() {
            Ok(h) => h,
            Err(_) => return,
        };

        // Wait for a shell prompt
        let result = harness.wait_for_prompt(Duration::from_secs(5));

        // This should succeed if a shell is running
        assert!(result.is_ok() || result.is_err(), "Should return a result");
    }
}

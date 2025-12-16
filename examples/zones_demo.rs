//! Example demonstrating OSC 133 semantic zone testing helpers.
//!
//! This example shows how to use the zones module to parse and work with
//! OSC 133 semantic zones for shell integration testing.

#[cfg(feature = "ipc")]
fn main() {
    use terminal_testlib::zones::{Osc133Parser, ZoneType};

    println!("OSC 133 Semantic Zone Parsing Demo\n");
    println!("===================================\n");

    // Create a parser
    let mut parser = Osc133Parser::new();

    // Simulate a shell command cycle with OSC 133 markers
    println!("Parsing shell command cycle...\n");

    let data = concat!(
        "\x1b]133;A\x07",           // Fresh line marker
        "$ ",                        // Prompt
        "\x1b]133;B\x07",           // Command start marker
        "echo 'Hello, World!'",      // Command text
        "\x1b]133;C\x07",           // Command executed marker
        "\nHello, World!\n",         // Command output
        "\x1b]133;D;0\x07",         // Command finished marker (exit code 0)
    )
    .as_bytes();

    parser.parse(data);

    // Display detected markers
    println!("Detected markers:");
    for (i, (marker, row, col)) in parser.markers().iter().enumerate() {
        println!("  {}: {:?} at row={}, col={}", i + 1, marker, row, col);
    }

    // Display detected zones
    println!("\nDetected zones:");
    let zones = parser.zones();
    for (i, zone) in zones.iter().enumerate() {
        println!(
            "  {}: {:?} zone from ({},{}) to ({},{})",
            i + 1,
            zone.zone_type,
            zone.start_row,
            zone.start_col,
            zone.end_row,
            zone.end_col
        );
        if let Some(exit_code) = zone.exit_code {
            println!("     Exit code: {}", exit_code);
        }
    }

    // Example 2: Multiple commands
    println!("\n\nParsing multiple commands...\n");

    parser.clear();

    let multi_data = concat!(
        // Command 1: Success
        "\x1b]133;A\x07$ ",
        "\x1b]133;B\x07ls -l",
        "\x1b]133;C\x07\ntotal 8\n-rw-r--r-- 1 user file.txt\n",
        "\x1b]133;D;0\x07",
        // Command 2: Failure
        "\x1b]133;A\x07$ ",
        "\x1b]133;B\x07false",
        "\x1b]133;C\x07\n",
        "\x1b]133;D;1\x07",
        // Command 3: Success
        "\x1b]133;A\x07$ ",
        "\x1b]133;B\x07pwd",
        "\x1b]133;C\x07\n/home/user\n",
        "\x1b]133;D;0\x07",
    )
    .as_bytes();

    parser.parse(multi_data);

    let zones = parser.zones();
    let prompts = zones.iter().filter(|z| z.zone_type == ZoneType::Prompt).count();
    let commands = zones.iter().filter(|z| z.zone_type == ZoneType::Command).count();
    let outputs = zones.iter().filter(|z| z.zone_type == ZoneType::Output).count();

    println!("Found {} zones:", zones.len());
    println!("  {} Prompt zones", prompts);
    println!("  {} Command zones", commands);
    println!("  {} Output zones", outputs);

    // Display exit codes
    println!("\nCommand results:");
    let output_zones: Vec<_> = zones
        .iter()
        .filter(|z| z.zone_type == ZoneType::Output)
        .collect();

    for (i, zone) in output_zones.iter().enumerate() {
        let status = match zone.exit_code {
            Some(0) => "SUCCESS",
            Some(code) => "FAILURE",
            None => "UNKNOWN",
        };
        println!(
            "  Command {}: {} (exit code: {})",
            i + 1,
            status,
            zone.exit_code.unwrap_or(-1)
        );
    }

    // Example 3: Working with zone text (would require actual grid data)
    println!("\n\nZone text extraction example:");
    println!("(In real usage with ScarabTestHarness, you would use zone_text() method)");
    println!("Example: harness.zone_text(&zone)?");

    println!("\n\nDemo complete!");
}

#[cfg(not(feature = "ipc"))]
fn main() {
    println!("This example requires the 'ipc' feature to be enabled.");
    println!("Run with: cargo run --example zones_demo --features ipc");
}

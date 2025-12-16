//! Example demonstrating UI region testing helpers.
//!
//! This example shows how to use the UI region testing utilities to verify
//! that content appears in the correct regions of a TUI application.
//!
//! # Running this example
//!
//! This example requires the `scarab` feature (which includes `ipc`):
//!
//! ```bash
//! cargo run --example ui_regions_test --features scarab
//! ```
//!
//! # Environment Setup
//!
//! Set the required environment variable:
//!
//! ```bash
//! export SCARAB_TEST_RTL=1
//! ```

#[cfg(feature = "scarab")]
use terminal_testlib::{
    regions::{RegionAnchor, RegionBounds, UiRegion, UiRegionTestExt, UiRegionTester},
    scarab::ScarabTestHarness,
};

#[cfg(feature = "scarab")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== UI Region Testing Demo ===\n");

    // Example 1: Basic region setup
    println!("Example 1: Basic Region Setup");
    println!("------------------------------");
    let tester = UiRegionTester::new(80, 24)
        .with_status_bar(1)
        .with_tab_bar(2)
        .with_left_sidebar(20);

    println!("Screen dimensions: {:?}", tester.screen_dimensions());
    println!("Regions: {:?}", tester.region_names());

    if let Some(status) = tester.region_bounds("status_bar") {
        println!("Status bar: row={}, col={}, width={}, height={}",
                 status.row, status.col, status.width, status.height);
    }

    if let Some(tabs) = tester.region_bounds("tab_bar") {
        println!("Tab bar: row={}, col={}, width={}, height={}",
                 tabs.row, tabs.col, tabs.width, tabs.height);
    }

    if let Some(sidebar) = tester.region_bounds("left_sidebar") {
        println!("Left sidebar: row={}, col={}, width={}, height={}",
                 sidebar.row, sidebar.col, sidebar.width, sidebar.height);
    }

    let content = tester.content_area();
    println!("Content area: row={}, col={}, width={}, height={}",
             content.row, content.col, content.width, content.height);
    println!();

    // Example 2: Position checking
    println!("Example 2: Position Checking");
    println!("-----------------------------");
    let test_positions = vec![
        (0, 10, "tab_bar"),
        (10, 10, "left_sidebar"),
        (10, 30, "content area"),
        (23, 40, "status_bar"),
    ];

    for (row, col, expected) in test_positions {
        let in_content = tester.is_in_content_area(row, col);
        let in_status = tester.is_in_region("status_bar", row, col);
        let in_tabs = tester.is_in_region("tab_bar", row, col);
        let in_sidebar = tester.is_in_region("left_sidebar", row, col);

        let actual = if in_status {
            "status_bar"
        } else if in_tabs {
            "tab_bar"
        } else if in_sidebar {
            "left_sidebar"
        } else if in_content {
            "content area"
        } else {
            "unknown"
        };

        let status = if actual == expected { "✓" } else { "✗" };
        println!("{} Position ({}, {}): expected '{}', got '{}'",
                 status, row, col, expected, actual);
    }
    println!();

    // Example 3: Custom regions
    println!("Example 3: Custom Regions");
    println!("-------------------------");
    let custom_tester = UiRegionTester::new(100, 30)
        .with_region(UiRegion {
            name: "header".to_string(),
            anchor: RegionAnchor::Top,
            size: 1,
        })
        .with_region(UiRegion {
            name: "notification".to_string(),
            anchor: RegionAnchor::Top,
            size: 3,
        })
        .with_status_bar(2)
        .with_right_sidebar(25);

    println!("Custom layout regions:");
    for name in custom_tester.region_names() {
        if let Some(bounds) = custom_tester.region_bounds(&name) {
            println!("  {}: row={}, col={}, {}x{}",
                     name, bounds.row, bounds.col, bounds.width, bounds.height);
        }
    }

    let custom_content = custom_tester.content_area();
    println!("  content: row={}, col={}, {}x{}",
             custom_content.row, custom_content.col,
             custom_content.width, custom_content.height);
    println!();

    // Example 4: Region intersection testing
    println!("Example 4: Region Intersection");
    println!("------------------------------");
    let region_a = RegionBounds::new(5, 5, 20, 10);
    let region_b = RegionBounds::new(10, 10, 20, 10);
    let region_c = RegionBounds::new(50, 50, 10, 10);

    println!("Region A: ({}, {}) {}x{}", region_a.row, region_a.col,
             region_a.width, region_a.height);
    println!("Region B: ({}, {}) {}x{}", region_b.row, region_b.col,
             region_b.width, region_b.height);
    println!("Region C: ({}, {}) {}x{}", region_c.row, region_c.col,
             region_c.width, region_c.height);
    println!();
    println!("A intersects B: {}", region_a.intersects(&region_b));
    println!("A intersects C: {}", region_a.intersects(&region_c));
    println!("B intersects C: {}", region_b.intersects(&region_c));
    println!();

    // Example 5: Testing with Scarab (if available)
    if ScarabTestHarness::is_enabled() {
        println!("Example 5: Testing with Scarab");
        println!("-------------------------------");

        match ScarabTestHarness::connect() {
            Ok(mut harness) => {
                println!("Connected to Scarab daemon");

                let tester = UiRegionTester::new(80, 24)
                    .with_status_bar(1)
                    .with_tab_bar(2);

                // Get dimensions
                let (cols, rows) = harness.dimensions();
                println!("Terminal dimensions: {}x{}", cols, rows);

                // Try to get region contents
                match harness.region_contents(&tester, "status_bar") {
                    Ok(contents) => {
                        println!("Status bar contents:");
                        println!("  '{}'", contents.replace('\n', "\\n"));
                    }
                    Err(e) => {
                        println!("Note: Could not get status bar contents: {}", e);
                    }
                }

                // Try to get content area
                match harness.content_area_contents(&tester) {
                    Ok(contents) => {
                        let lines: Vec<&str> = contents.lines().collect();
                        println!("Content area has {} lines", lines.len());
                        if !lines.is_empty() {
                            println!("First line: '{}'", lines[0]);
                        }
                    }
                    Err(e) => {
                        println!("Note: Could not get content area: {}", e);
                    }
                }

                // Test assertion methods
                println!("\nTesting assertions:");

                // This should succeed if "ERROR" is not in status bar
                match harness.assert_not_in_region(&tester, "status_bar", "ERROR") {
                    Ok(_) => println!("✓ Status bar does not contain 'ERROR'"),
                    Err(e) => println!("✗ Assertion failed: {}", e),
                }

                // Test resize
                println!("\nTesting resize:");
                let mut resize_tester = tester.clone();
                match harness.verify_resize(&mut resize_tester, 100, 30) {
                    Ok(_) => {
                        println!("✓ Resize to 100x30 succeeded");
                        let new_content = resize_tester.content_area();
                        println!("  New content area: row={}, col={}, {}x{}",
                                 new_content.row, new_content.col,
                                 new_content.width, new_content.height);
                    }
                    Err(e) => println!("Note: Resize test skipped: {}", e),
                }

                println!("Scarab integration test completed");
            }
            Err(e) => {
                println!("Note: Could not connect to Scarab daemon: {}", e);
                println!("This is expected if scarab-daemon is not running");
            }
        }
    } else {
        println!("Example 5: Scarab Testing (Disabled)");
        println!("-------------------------------------");
        println!("Set SCARAB_TEST_RTL=1 to enable Scarab integration");
    }
    println!();

    println!("=== Demo completed successfully ===");
    Ok(())
}

#[cfg(not(feature = "scarab"))]
fn main() {
    eprintln!("This example requires the 'scarab' feature.");
    eprintln!("Run with: cargo run --example ui_regions_test --features scarab");
    std::process::exit(1);
}

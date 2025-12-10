//! Example demonstrating theme and color palette verification.
//!
//! This example shows how to use the theme module to verify that terminals
//! correctly apply color palettes and themes.
//!
//! # Usage
//!
//! This requires a running scarab-daemon with proper setup. Enable the test mode:
//!
//! ```bash
//! export SCARAB_TEST_RTL=1
//! cargo run --example theme_test --features scarab
//! ```

#[cfg(feature = "scarab")]
use ratatui_testlib::{
    scarab::ScarabTestHarness,
    theme::{AnsiColor, ColorPalette, ThemeTestExt},
};
use std::time::Duration;

#[cfg(feature = "scarab")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Theme and Color Palette Verification Demo ===\n");

    // Connect to Scarab daemon
    let mut harness = match ScarabTestHarness::connect() {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to connect to Scarab daemon: {}", e);
            eprintln!("\nMake sure:");
            eprintln!("  1. scarab-daemon is running");
            eprintln!("  2. SCARAB_TEST_RTL=1 is set");
            return Ok(());
        }
    };

    println!("✓ Connected to Scarab daemon\n");

    // Wait for shell prompt
    println!("Waiting for shell prompt...");
    harness.wait_for_prompt(Duration::from_secs(5))?;
    println!("✓ Shell ready\n");

    // Test 1: Print colored text
    println!("Test 1: Sending ANSI colored text");
    println!("---------------------------------------");
    harness.send_input("\x1b[31mRed\x1b[0m ")?;
    harness.send_input("\x1b[32mGreen\x1b[0m ")?;
    harness.send_input("\x1b[33mYellow\x1b[0m ")?;
    harness.send_input("\x1b[34mBlue\x1b[0m\n")?;

    std::thread::sleep(Duration::from_millis(100));
    println!("✓ Colored text sent\n");

    // Test 2: Verify colors using different palettes
    println!("Test 2: Verifying colors with different palettes");
    println!("---------------------------------------");

    let palettes = vec![
        ColorPalette::slime(),
        ColorPalette::dracula(),
        ColorPalette::nord(),
        ColorPalette::monokai(),
    ];

    for palette in &palettes {
        println!("  Checking '{}' palette:", palette.name);
        println!("    - ANSI Red:    0x{:08X}", palette.ansi_color(AnsiColor::Red));
        println!(
            "    - ANSI Green:  0x{:08X}",
            palette.ansi_color(AnsiColor::Green)
        );
        println!(
            "    - ANSI Yellow: 0x{:08X}",
            palette.ansi_color(AnsiColor::Yellow)
        );
        println!(
            "    - ANSI Blue:   0x{:08X}",
            palette.ansi_color(AnsiColor::Blue)
        );
        println!();
    }

    // Test 3: Capture cell colors
    println!("Test 3: Capturing cell colors at specific positions");
    println!("---------------------------------------");

    match harness.capture_cell_colors(0, 0) {
        Ok((fg, bg)) => {
            println!("  Cell (0, 0):");
            println!("    - Foreground: 0x{:08X}", fg);
            println!("    - Background: 0x{:08X}", bg);
        }
        Err(e) => {
            println!("  Could not capture cell colors: {}", e);
        }
    }
    println!();

    // Test 4: Scan colors in a region
    println!("Test 4: Scanning colors in a region");
    println!("---------------------------------------");

    match harness.scan_colors_in_region(0, 0, 5, 40) {
        Ok(scan) => {
            println!("  Region (0,0) to (5,40):");
            println!("    - Cells scanned: {}", scan.cells_scanned);
            println!(
                "    - Unique foreground colors: {}",
                scan.unique_foreground_count()
            );
            println!(
                "    - Unique background colors: {}",
                scan.unique_background_count()
            );
            println!();
            println!("  Foreground colors found:");
            for (i, color) in scan.foreground_colors.iter().enumerate() {
                println!("    {}. 0x{:08X}", i + 1, color);
            }
            println!();
            println!("  Background colors found:");
            for (i, color) in scan.background_colors.iter().enumerate() {
                println!("    {}. 0x{:08X}", i + 1, color);
            }
        }
        Err(e) => {
            println!("  Could not scan region: {}", e);
        }
    }
    println!();

    // Test 5: Test color assertion (demonstrative)
    println!("Test 5: Color assertion examples");
    println!("---------------------------------------");

    // Try to get cell colors and verify them
    match harness.cell_foreground(0, 0) {
        Ok(fg) => {
            println!("  Cell (0, 0) foreground: 0x{:08X}", fg);

            // Try exact match with different palettes
            let slime = ColorPalette::slime();
            let dracula = ColorPalette::dracula();

            println!("  Checking if it matches ANSI colors:");
            for ansi in [
                AnsiColor::Red,
                AnsiColor::Green,
                AnsiColor::Blue,
                AnsiColor::White,
            ] {
                let slime_match = slime.matches_ansi(fg, ansi);
                let dracula_match = dracula.matches_ansi(fg, ansi);

                if slime_match || dracula_match {
                    println!(
                        "    {:?}: {} (slime) | {} (dracula)",
                        ansi,
                        if slime_match { "✓" } else { "✗" },
                        if dracula_match { "✓" } else { "✗" }
                    );
                }
            }
        }
        Err(e) => {
            println!("  Could not get cell foreground: {}", e);
        }
    }
    println!();

    // Test 6: Compare theme configurations
    println!("Test 6: Theme configuration comparison");
    println!("---------------------------------------");

    let slime = ColorPalette::slime();
    let dracula = ColorPalette::dracula();

    println!("  Slime theme:");
    println!("    - Background: 0x{:08X}", slime.background);
    println!("    - Foreground: 0x{:08X}", slime.foreground);
    println!("    - Cursor:     0x{:08X}", slime.cursor);
    println!("    - Selection:  0x{:08X}", slime.selection);
    println!();

    println!("  Dracula theme:");
    println!("    - Background: 0x{:08X}", dracula.background);
    println!("    - Foreground: 0x{:08X}", dracula.foreground);
    println!("    - Cursor:     0x{:08X}", dracula.cursor);
    println!("    - Selection:  0x{:08X}", dracula.selection);
    println!();

    println!("=== Demo Complete ===");
    println!("\nNote: To properly verify colors, you need to configure");
    println!("      scarab-daemon to expose cell attributes via shared memory.");

    Ok(())
}

#[cfg(not(feature = "scarab"))]
fn main() {
    eprintln!("This example requires the 'scarab' feature.");
    eprintln!("Run with: cargo run --example theme_test --features scarab");
}

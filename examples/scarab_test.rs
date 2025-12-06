//! Example demonstrating Scarab-specific testing.
//!
//! This example shows how to use the `scarab` module to test Scarab terminal
//! emulator integration without reimplementing IPC/shared-memory plumbing.
//!
//! # Prerequisites
//!
//! 1. A running scarab-daemon that:
//!    - Listens on `/tmp/scarab-daemon.sock`
//!    - Exposes terminal state via `/scarab_shm_v1`
//!
//! 2. Environment variable set:
//!    ```bash
//!    export SCARAB_TEST_RTL=1
//!    ```
//!
//! # Running this example
//!
//! ```bash
//! # Start scarab-daemon first
//! scarab-daemon &
//!
//! # Run the example
//! SCARAB_TEST_RTL=1 cargo run --example scarab_test --features scarab
//! ```

#[cfg(feature = "scarab")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use std::time::Duration;

    use ratatui_testlib::ipc::IpcError;
    use ratatui_testlib::scarab::{ScarabConfig, ScarabTestHarness};

    println!("Scarab Test Example");
    println!("===================\n");

    // Check if Scarab testing is enabled
    if !ScarabTestHarness::is_enabled() {
        println!("Scarab testing is disabled.");
        println!("Set SCARAB_TEST_RTL=1 to enable Scarab testing.\n");
        println!("Example:");
        println!("  SCARAB_TEST_RTL=1 cargo run --example scarab_test --features scarab\n");
        return Ok(());
    }

    println!("Scarab testing is enabled!\n");

    // Show default configuration
    let default_config = ScarabConfig::default();
    println!("Default Configuration:");
    println!("  Socket: {:?}", default_config.socket_path);
    println!("  Shared memory: {}", default_config.shm_path);
    println!("  Image shm: {:?}", default_config.image_shm_path);
    println!("  Dimensions: {:?}", default_config.dimensions);
    println!("  Prompt patterns: {:?}", default_config.prompt_patterns);
    println!();

    // Try to connect to Scarab daemon
    match ScarabTestHarness::connect() {
        Ok(mut harness) => {
            println!("Connected to scarab-daemon successfully!\n");

            // Get terminal dimensions
            let (cols, rows) = harness.dimensions();
            println!("Terminal size: {}x{}", cols, rows);

            // Get cursor position
            let (row, col) = harness.cursor_position()?;
            println!("Cursor position: row={}, col={}", row, col);

            // Read initial grid contents
            let grid = harness.grid_contents()?;
            println!("\nInitial grid contents:");
            println!("---");
            for line in grid.lines().take(5) {
                println!("{}", line);
            }
            if grid.lines().count() > 5 {
                println!("... ({} more lines)", grid.lines().count() - 5);
            }
            println!("---\n");

            // Wait for prompt
            println!("Waiting for shell prompt...");
            match harness.wait_for_prompt(Duration::from_secs(5)) {
                Ok(()) => println!("Prompt detected!"),
                Err(IpcError::Timeout(_)) => println!("Timeout waiting for prompt (continuing...)"),
                Err(e) => println!("Error: {}", e),
            }

            // Send a test command
            println!("\nSending test command: echo 'Hello from Scarab test!'");
            harness.send_input("echo 'Hello from Scarab test!'\n")?;

            // Wait for the output
            println!("Waiting for output...");
            match harness.wait_for_text("Hello from Scarab test!", Duration::from_secs(5)) {
                Ok(()) => {
                    println!("Output received!\n");

                    // Read updated grid
                    let grid = harness.grid_contents()?;
                    println!("Updated grid contents:");
                    println!("---");
                    for line in grid.lines().take(10) {
                        println!("{}", line);
                    }
                    println!("---\n");

                    // Assert the output
                    harness.assert_contains("Hello from Scarab test!")?;
                    println!("Assertion passed!");
                }
                Err(IpcError::Timeout(duration)) => {
                    println!("Timeout after {:?} waiting for output.", duration);
                    println!("This might mean the daemon isn't processing input.\n");
                }
                Err(e) => {
                    println!("Error waiting for output: {}", e);
                }
            }

            println!("\nScarab test completed successfully!");
        }
        Err(IpcError::SocketNotFound(path)) => {
            println!("Scarab daemon socket not found at: {}", path.display());
            println!("\nMake sure scarab-daemon is running:");
            println!("  scarab-daemon &");
        }
        Err(IpcError::SharedMemoryNotFound(path)) => {
            println!("Scarab shared memory not found at: {}", path);
            println!("\nMake sure scarab-daemon creates shared memory.");
        }
        Err(IpcError::InvalidData(msg)) => {
            println!("Invalid shared memory format: {}", msg);
            println!("\nThis may indicate a protocol version mismatch.");
            println!("Scarab expects magic: 0x5343_5241 (\"SCRA\")");
        }
        Err(e) => {
            println!("Failed to connect to Scarab daemon: {}", e);
        }
    }

    Ok(())
}

#[cfg(not(feature = "scarab"))]
fn main() {
    println!("This example requires the 'scarab' feature.");
    println!("Run with: cargo run --example scarab_test --features scarab");
}

//! Example demonstrating seqlock verification for detecting torn reads.
//!
//! This example shows how to use the seqlock verification helpers to detect
//! torn reads in shared memory accessed by split-process terminal applications.
//!
//! # Prerequisites
//!
//! This example requires a running Scarab daemon with shared memory set up.
//! Set the environment variable to enable testing:
//!
//! ```bash
//! export SCARAB_TEST_RTL=1
//! ```
//!
//! # Usage
//!
//! ```bash
//! cargo run --example seqlock_verification --features scarab
//! ```

#[cfg(feature = "scarab")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    use ratatui_testlib::scarab::{ScarabTestHarness, SeqlockTestExt};
    use std::time::Duration;

    println!("Seqlock Verification Example");
    println!("============================\n");

    // Check if testing is enabled
    if !ScarabTestHarness::is_enabled() {
        eprintln!("Error: SCARAB_TEST_RTL environment variable not set");
        eprintln!("Please run: export SCARAB_TEST_RTL=1");
        return Ok(());
    }

    // Connect to the Scarab daemon
    println!("Connecting to Scarab daemon...");
    let mut harness = match ScarabTestHarness::connect() {
        Ok(h) => {
            println!("Connected successfully!\n");
            h
        }
        Err(e) => {
            eprintln!("Failed to connect: {}", e);
            eprintln!("Make sure the Scarab daemon is running.");
            return Err(e.into());
        }
    };

    // Example 1: Perform a single synchronized read
    println!("Example 1: Synchronized Read");
    println!("-----------------------------");
    println!("Performing a synchronized read with torn-read protection...");

    let grid = harness.synchronized_read(|shm| shm.grid_contents())?;
    println!("Successfully read grid contents (length: {} chars)", grid.len());
    println!();

    // Example 2: Check if write is currently in progress
    println!("Example 2: Write-in-Progress Detection");
    println!("---------------------------------------");
    let verifier = harness.seqlock_verifier();
    let shm = harness.shared_memory();

    if verifier.is_write_in_progress(shm.as_daemon_shm()) {
        println!("Write currently in progress (sequence number is odd)");
    } else {
        println!("No write in progress (sequence number is even)");
    }
    println!();

    // Example 3: Run continuous verification
    println!("Example 3: Continuous Verification");
    println!("-----------------------------------");
    println!("Running seqlock verification for 3 seconds...");
    println!("(Send some input to the terminal to trigger updates)\n");

    let report = harness.verify_seqlock(Duration::from_secs(3))?;

    println!("Verification Report:");
    println!("  Total reads:              {}", report.total_reads);
    println!("  Torn read detections:     {}", report.torn_read_detections);
    println!("  Odd sequence detections:  {}", report.odd_sequence_detections);
    println!("  Max retry count:          {}", report.max_retry_count);
    println!("  Average retry count:      {:.3}", report.avg_retry_count);
    println!("  Retry percentage:         {:.2}%", report.retry_percentage());
    println!();

    // Provide interpretation
    if report.has_torn_reads() {
        println!("Analysis: Torn reads detected!");
        println!("  This indicates that data was changing during read operations.");
        println!(
            "  {}% of reads required retries to ensure consistency.",
            report.retry_percentage()
        );
    } else {
        println!("Analysis: No torn reads detected.");
        println!("  All reads completed without detecting concurrent writes.");
    }

    if report.has_odd_sequences() {
        println!(
            "  {} times the sequence number was odd (write in progress).",
            report.odd_sequence_detections
        );
    }

    println!();

    // Example 4: Multiple synchronized reads in succession
    println!("Example 4: Multiple Synchronized Reads");
    println!("---------------------------------------");
    println!("Performing 10 synchronized reads...");

    for i in 1..=10 {
        let grid = harness.synchronized_read(|shm| shm.grid_contents())?;
        println!("Read {}: {} chars", i, grid.len());
    }

    println!("Completed all reads successfully.\n");

    // Example 5: Custom verification duration
    println!("Example 5: Custom Verification Parameters");
    println!("------------------------------------------");
    println!("Running verification for 1 second with custom polling...");

    let report = harness.verify_seqlock(Duration::from_secs(1))?;

    println!("Short verification report:");
    println!("  Reads performed: {}", report.total_reads);
    println!("  Torn reads:      {}", report.torn_read_detections);
    println!();

    // Summary
    println!("Summary");
    println!("-------");
    println!("Seqlock verification helps detect torn reads in shared memory");
    println!("by checking sequence numbers before and after read operations.");
    println!();
    println!("Key benefits:");
    println!("  - Detects concurrent writes during reads");
    println!("  - Automatically retries on torn reads");
    println!("  - Provides statistics for analysis");
    println!("  - Zero-copy reads when no writes are in progress");

    Ok(())
}

#[cfg(not(feature = "scarab"))]
fn main() {
    eprintln!("This example requires the 'scarab' feature.");
    eprintln!("Run with: cargo run --example seqlock_verification --features scarab");
}

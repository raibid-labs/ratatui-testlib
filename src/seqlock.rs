//! Seqlock verification helpers for detecting torn reads in shared memory.
//!
//! This module provides utilities for verifying seqlock-based synchronization in
//! shared memory systems. Seqlocks are a reader-writer synchronization primitive
//! where writers increment a sequence number (making it odd during writes, even
//! when complete), and readers validate by checking the sequence number before
//! and after reading.
//!
//! # Overview
//!
//! Seqlocks provide lock-free reads with retry-based consistency guarantees:
//! - Writers increment sequence counter before/after writes (odd = in-progress)
//! - Readers check sequence parity and consistency across read operations
//! - Torn reads are detected when sequence changes during read
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "ipc")]
//! # {
//! use std::time::Duration;
//! use terminal_testlib::seqlock::SeqlockVerifier;
//! use terminal_testlib::ipc::{DaemonSharedMemory, IpcResult};
//!
//! # fn test() -> IpcResult<()> {
//! let mut shm = DaemonSharedMemory::open("/my_shm")?;
//! let mut verifier = SeqlockVerifier::new();
//!
//! // Perform a synchronized read with automatic retry on torn reads
//! let (grid, retries) = verifier.synchronized_read(&mut shm, |shm| {
//!     shm.grid_contents()
//! })?;
//!
//! println!("Read grid with {} retries", retries);
//!
//! // Run continuous verification for 5 seconds
//! let report = verifier.verify_seqlock_pattern(
//!     &mut shm,
//!     Duration::from_secs(5),
//!     Duration::from_millis(10)
//! )?;
//!
//! println!("Torn reads detected: {}", report.torn_read_detections);
//! println!("Average retries: {:.2}", report.avg_retry_count);
//! # Ok(())
//! # }
//! # }
//! ```

use std::sync::atomic::{AtomicU32, Ordering};
use std::time::{Duration, Instant};

use crate::ipc::{DaemonSharedMemory, IpcError, IpcResult};

/// Report from seqlock verification.
///
/// Contains statistics about sequence number changes, torn read detections,
/// and retry behavior during verification.
#[derive(Debug, Clone, Default)]
pub struct SeqlockReport {
    /// Total number of read attempts.
    pub total_reads: u64,

    /// Number of torn reads detected (had to retry).
    pub torn_read_detections: u64,

    /// Number of times sequence was odd (write in progress).
    pub odd_sequence_detections: u64,

    /// Maximum number of retries for a single read.
    pub max_retry_count: u32,

    /// Average retry count across all reads.
    pub avg_retry_count: f64,
}

impl SeqlockReport {
    /// Create a new empty report.
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if any torn reads were detected.
    pub fn has_torn_reads(&self) -> bool {
        self.torn_read_detections > 0
    }

    /// Check if any odd sequences were detected (writes in progress).
    pub fn has_odd_sequences(&self) -> bool {
        self.odd_sequence_detections > 0
    }

    /// Calculate the percentage of reads that required retries.
    pub fn retry_percentage(&self) -> f64 {
        if self.total_reads == 0 {
            0.0
        } else {
            (self.torn_read_detections as f64 / self.total_reads as f64) * 100.0
        }
    }
}

/// Seqlock verifier for detecting torn reads.
///
/// This verifier tracks sequence number changes and detects torn reads
/// by validating sequence consistency before and after read operations.
#[derive(Debug)]
pub struct SeqlockVerifier {
    last_seq: AtomicU32,
    report: SeqlockReport,
}

impl SeqlockVerifier {
    /// Create a new verifier.
    pub fn new() -> Self {
        Self {
            last_seq: AtomicU32::new(0),
            report: SeqlockReport::default(),
        }
    }

    /// Perform a synchronized read, retrying on torn reads.
    ///
    /// This method implements the seqlock read protocol:
    /// 1. Read sequence number and ensure it's even (no write in progress)
    /// 2. Perform the read operation
    /// 3. Read sequence number again and verify it hasn't changed
    /// 4. Retry if torn read detected
    ///
    /// Returns the result along with the number of retries needed.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "ipc")]
    /// # {
    /// use terminal_testlib::seqlock::SeqlockVerifier;
    /// use terminal_testlib::ipc::{DaemonSharedMemory, IpcResult};
    ///
    /// # fn test() -> IpcResult<()> {
    /// let mut shm = DaemonSharedMemory::open("/my_shm")?;
    /// let mut verifier = SeqlockVerifier::new();
    ///
    /// let (grid, retries) = verifier.synchronized_read(&mut shm, |shm| {
    ///     shm.grid_contents()
    /// })?;
    ///
    /// if retries > 0 {
    ///     println!("Detected torn read, retried {} times", retries);
    /// }
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    pub fn synchronized_read<F, T>(
        &mut self,
        shm: &mut DaemonSharedMemory,
        reader: F,
    ) -> IpcResult<(T, u32)>
    where
        F: Fn(&DaemonSharedMemory) -> IpcResult<T>,
    {
        const MAX_RETRIES: u32 = 100;
        let mut retry_count = 0;

        loop {
            // Read sequence number before
            let seq_before = shm.raw_sequence_number();

            // If sequence is odd, a write is in progress
            if seq_before & 1 != 0 {
                self.report.odd_sequence_detections += 1;
                retry_count += 1;

                if retry_count >= MAX_RETRIES {
                    return Err(IpcError::InvalidData(
                        format!("Seqlock retry limit exceeded after {} attempts", MAX_RETRIES)
                    ));
                }

                // Spin briefly and retry
                std::thread::yield_now();
                continue;
            }

            // Perform the read
            let result = reader(shm)?;

            // Read sequence number after and ensure shared memory is updated
            shm.refresh()?;
            let seq_after = shm.raw_sequence_number();

            // Check if sequence changed during read (torn read)
            if seq_before != seq_after {
                self.report.torn_read_detections += 1;
                retry_count += 1;

                if retry_count >= MAX_RETRIES {
                    return Err(IpcError::InvalidData(
                        format!("Seqlock retry limit exceeded after {} attempts", MAX_RETRIES)
                    ));
                }

                // Retry the read
                continue;
            }

            // Success - update statistics
            self.report.total_reads += 1;
            if retry_count > 0 {
                self.report.max_retry_count = self.report.max_retry_count.max(retry_count);
            }

            // Update average retry count
            let total_retries = self.report.torn_read_detections + self.report.odd_sequence_detections;
            self.report.avg_retry_count = if self.report.total_reads > 0 {
                total_retries as f64 / self.report.total_reads as f64
            } else {
                0.0
            };

            // Store last seen sequence
            self.last_seq.store(seq_after, Ordering::SeqCst);

            return Ok((result, retry_count));
        }
    }

    /// Check if the current state shows signs of a torn read.
    ///
    /// Returns true if sequence number is odd (write in progress).
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "ipc")]
    /// # {
    /// use terminal_testlib::seqlock::SeqlockVerifier;
    /// use terminal_testlib::ipc::DaemonSharedMemory;
    ///
    /// # fn test() -> terminal_testlib::ipc::IpcResult<()> {
    /// let shm = DaemonSharedMemory::open("/my_shm")?;
    /// let verifier = SeqlockVerifier::new();
    ///
    /// if verifier.is_write_in_progress(&shm) {
    ///     println!("Write currently in progress, sequence is odd");
    /// }
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    pub fn is_write_in_progress(&self, shm: &DaemonSharedMemory) -> bool {
        let seq = shm.raw_sequence_number();
        seq & 1 != 0
    }

    /// Verify the seqlock pattern is being used correctly over a duration.
    ///
    /// Watches for sequence number changes and verifies they follow the seqlock
    /// protocol (even sequences indicate consistent state). Collects statistics
    /// about torn reads, retry behavior, and odd sequence detections.
    ///
    /// # Parameters
    ///
    /// - `shm`: Shared memory to monitor
    /// - `duration`: How long to run verification
    /// - `poll_interval`: How frequently to check sequence numbers
    ///
    /// # Returns
    ///
    /// A report containing statistics about the verification run.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "ipc")]
    /// # {
    /// use std::time::Duration;
    /// use terminal_testlib::seqlock::SeqlockVerifier;
    /// use terminal_testlib::ipc::DaemonSharedMemory;
    ///
    /// # fn test() -> terminal_testlib::ipc::IpcResult<()> {
    /// let mut shm = DaemonSharedMemory::open("/my_shm")?;
    /// let mut verifier = SeqlockVerifier::new();
    ///
    /// // Monitor for 10 seconds, polling every 5ms
    /// let report = verifier.verify_seqlock_pattern(
    ///     &mut shm,
    ///     Duration::from_secs(10),
    ///     Duration::from_millis(5)
    /// )?;
    ///
    /// println!("Total reads: {}", report.total_reads);
    /// println!("Torn reads: {}", report.torn_read_detections);
    /// println!("Retry rate: {:.2}%", report.retry_percentage());
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    pub fn verify_seqlock_pattern(
        &mut self,
        shm: &mut DaemonSharedMemory,
        duration: Duration,
        poll_interval: Duration,
    ) -> IpcResult<SeqlockReport> {
        let start = Instant::now();

        while start.elapsed() < duration {
            // Perform a synchronized read of the grid contents
            let (_grid, retries) = self.synchronized_read(shm, |shm| {
                shm.grid_contents()
            })?;

            // Track if we had to retry
            if retries > 0 {
                self.report.max_retry_count = self.report.max_retry_count.max(retries);
            }

            // Sleep for the poll interval
            std::thread::sleep(poll_interval);
        }

        Ok(self.report.clone())
    }

    /// Get the current verification report.
    ///
    /// Returns a reference to the accumulated statistics.
    pub fn report(&self) -> &SeqlockReport {
        &self.report
    }

    /// Reset the verification report.
    ///
    /// Clears all accumulated statistics, useful for starting a fresh
    /// verification run.
    pub fn reset(&mut self) {
        self.report = SeqlockReport::default();
        self.last_seq.store(0, Ordering::SeqCst);
    }
}

impl Default for SeqlockVerifier {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_seqlock_report_default() {
        let report = SeqlockReport::default();
        assert_eq!(report.total_reads, 0);
        assert_eq!(report.torn_read_detections, 0);
        assert_eq!(report.odd_sequence_detections, 0);
        assert_eq!(report.max_retry_count, 0);
        assert_eq!(report.avg_retry_count, 0.0);
    }

    #[test]
    fn test_seqlock_report_new() {
        let report = SeqlockReport::new();
        assert_eq!(report.total_reads, 0);
        assert!(!report.has_torn_reads());
        assert!(!report.has_odd_sequences());
    }

    #[test]
    fn test_seqlock_report_has_torn_reads() {
        let mut report = SeqlockReport::default();
        assert!(!report.has_torn_reads());

        report.torn_read_detections = 1;
        assert!(report.has_torn_reads());
    }

    #[test]
    fn test_seqlock_report_has_odd_sequences() {
        let mut report = SeqlockReport::default();
        assert!(!report.has_odd_sequences());

        report.odd_sequence_detections = 1;
        assert!(report.has_odd_sequences());
    }

    #[test]
    fn test_seqlock_report_retry_percentage() {
        let mut report = SeqlockReport::default();
        assert_eq!(report.retry_percentage(), 0.0);

        report.total_reads = 100;
        report.torn_read_detections = 25;
        assert_eq!(report.retry_percentage(), 25.0);

        report.total_reads = 200;
        report.torn_read_detections = 10;
        assert_eq!(report.retry_percentage(), 5.0);
    }

    #[test]
    fn test_verifier_new() {
        let verifier = SeqlockVerifier::new();
        assert_eq!(verifier.last_seq.load(Ordering::SeqCst), 0);
        assert_eq!(verifier.report.total_reads, 0);
    }

    #[test]
    fn test_verifier_default() {
        let verifier = SeqlockVerifier::default();
        assert_eq!(verifier.last_seq.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_verifier_reset() {
        let mut verifier = SeqlockVerifier::new();
        verifier.report.total_reads = 100;
        verifier.report.torn_read_detections = 10;
        verifier.last_seq.store(42, Ordering::SeqCst);

        verifier.reset();

        assert_eq!(verifier.report.total_reads, 0);
        assert_eq!(verifier.report.torn_read_detections, 0);
        assert_eq!(verifier.last_seq.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_report_access() {
        let verifier = SeqlockVerifier::new();
        let report = verifier.report();
        assert_eq!(report.total_reads, 0);
    }
}

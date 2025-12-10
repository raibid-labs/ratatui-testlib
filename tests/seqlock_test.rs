//! Integration tests for seqlock verification helpers.

#[cfg(feature = "ipc")]
mod tests {
    use ratatui_testlib::seqlock::{SeqlockReport, SeqlockVerifier};

    #[test]
    fn test_seqlock_report_construction() {
        let report = SeqlockReport::new();
        assert_eq!(report.total_reads, 0);
        assert_eq!(report.torn_read_detections, 0);
        assert_eq!(report.odd_sequence_detections, 0);
        assert_eq!(report.max_retry_count, 0);
        assert_eq!(report.avg_retry_count, 0.0);
        assert!(!report.has_torn_reads());
        assert!(!report.has_odd_sequences());
        assert_eq!(report.retry_percentage(), 0.0);
    }

    #[test]
    fn test_seqlock_report_with_torn_reads() {
        let mut report = SeqlockReport::new();
        report.total_reads = 100;
        report.torn_read_detections = 10;

        assert!(report.has_torn_reads());
        assert_eq!(report.retry_percentage(), 10.0);
    }

    #[test]
    fn test_seqlock_report_with_odd_sequences() {
        let mut report = SeqlockReport::new();
        report.odd_sequence_detections = 5;

        assert!(report.has_odd_sequences());
    }

    #[test]
    fn test_seqlock_report_retry_percentage_edge_cases() {
        let mut report = SeqlockReport::new();

        // Zero total reads
        assert_eq!(report.retry_percentage(), 0.0);

        // All reads required retries
        report.total_reads = 50;
        report.torn_read_detections = 50;
        assert_eq!(report.retry_percentage(), 100.0);

        // Half required retries
        report.total_reads = 100;
        report.torn_read_detections = 50;
        assert_eq!(report.retry_percentage(), 50.0);
    }

    #[test]
    fn test_verifier_construction() {
        let verifier = SeqlockVerifier::new();
        let report = verifier.report();
        assert_eq!(report.total_reads, 0);
    }

    #[test]
    fn test_verifier_default() {
        let verifier = SeqlockVerifier::default();
        let report = verifier.report();
        assert_eq!(report.total_reads, 0);
    }

    #[test]
    fn test_verifier_reset() {
        let mut verifier = SeqlockVerifier::new();

        // After reset, report should be cleared
        verifier.reset();

        let report = verifier.report();
        assert_eq!(report.total_reads, 0);
        assert_eq!(report.torn_read_detections, 0);
    }

    #[test]
    fn test_report_clone() {
        let mut report1 = SeqlockReport::new();
        report1.total_reads = 42;
        report1.torn_read_detections = 5;

        let report2 = report1.clone();
        assert_eq!(report2.total_reads, 42);
        assert_eq!(report2.torn_read_detections, 5);
    }

    #[test]
    fn test_report_debug() {
        let report = SeqlockReport::new();
        let debug_str = format!("{:?}", report);
        assert!(debug_str.contains("SeqlockReport"));
    }

    #[test]
    fn test_verifier_debug() {
        let verifier = SeqlockVerifier::new();
        let debug_str = format!("{:?}", verifier);
        assert!(debug_str.contains("SeqlockVerifier"));
    }

    #[test]
    fn test_report_statistics_accuracy() {
        let mut report = SeqlockReport::new();

        // Simulate multiple read cycles with different retry patterns
        report.total_reads = 1000;
        report.torn_read_detections = 50;  // 5% of reads had torn reads
        report.odd_sequence_detections = 25; // 2.5% encountered writes in progress
        report.max_retry_count = 3;
        report.avg_retry_count = 0.075; // (50 + 25) / 1000 = 0.075

        assert_eq!(report.retry_percentage(), 5.0);
        assert!(report.has_torn_reads());
        assert!(report.has_odd_sequences());
        assert_eq!(report.max_retry_count, 3);
    }

    #[test]
    fn test_verifier_report_access() {
        let verifier = SeqlockVerifier::new();
        let report = verifier.report();

        // Verify we can access all fields
        assert_eq!(report.total_reads, 0);
        assert_eq!(report.torn_read_detections, 0);
        assert_eq!(report.odd_sequence_detections, 0);
        assert_eq!(report.max_retry_count, 0);
        assert_eq!(report.avg_retry_count, 0.0);
    }

    // Note: Tests that require actual shared memory access should be in
    // integration tests with a real daemon running. These unit tests verify
    // the internal logic and data structures.
}

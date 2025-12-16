//! Integration tests for the audit module.

use terminal_testlib::audit::{
    HarnessType, PlaceholderPattern, ScaffoldConfig, TestAuditor,
};
use std::fs;
use tempfile::TempDir;

#[test]
fn test_audit_workflow_end_to_end() -> std::io::Result<()> {
    // Setup: Create test files with placeholders
    let temp_dir = TempDir::new()?;
    let test_file = temp_dir.path().join("placeholder_tests.rs");

    fs::write(
        &test_file,
        r#"
#[test]
fn test_daemon_terminal_processing() {
    assert!(true);
}

#[test]
fn test_shared_memory_access() {
    assert_eq!(1, 1);
}

#[test]
fn test_sixel_graphics_rendering() {
    todo!();
}

#[test]
fn test_real_implementation() {
    let x = compute_value();
    assert_eq!(x, 42);
}
"#,
    )?;

    // Step 1: Find placeholders
    let placeholders = TestAuditor::find_placeholders_in_file(&test_file)?;
    assert_eq!(placeholders.len(), 3); // Only placeholder tests, not the real one

    // Step 2: Verify patterns
    assert!(placeholders
        .iter()
        .any(|p| p.pattern == PlaceholderPattern::AssertTrue));
    assert!(placeholders
        .iter()
        .any(|p| p.pattern == PlaceholderPattern::TrivialEquality));
    assert!(placeholders
        .iter()
        .any(|p| p.pattern == PlaceholderPattern::TodoMacro));

    // Step 3: Verify function names
    assert!(placeholders
        .iter()
        .any(|p| p.function_name == "test_daemon_terminal_processing"));
    assert!(placeholders
        .iter()
        .any(|p| p.function_name == "test_shared_memory_access"));
    assert!(placeholders
        .iter()
        .any(|p| p.function_name == "test_sixel_graphics_rendering"));

    // Step 4: Verify subject inference
    let daemon_test = placeholders
        .iter()
        .find(|p| p.function_name == "test_daemon_terminal_processing")
        .unwrap();
    assert_eq!(daemon_test.inferred_subject, "daemon terminal processing");

    let memory_test = placeholders
        .iter()
        .find(|p| p.function_name == "test_shared_memory_access")
        .unwrap();
    assert_eq!(memory_test.inferred_subject, "shared memory access");

    // Step 5: Generate templates
    let config = ScaffoldConfig::default();
    let template = TestAuditor::generate_template(daemon_test, &config);
    assert!(template.contains("test_daemon_terminal_processing"));
    assert!(template.contains("TuiTestHarness"));
    assert!(template.contains("daemon terminal processing"));

    // Step 6: Generate full scaffolded file
    let scaffolded = TestAuditor::scaffold_test_file(&placeholders, &config);
    assert!(scaffolded.contains("Scaffolded test file"));
    assert!(scaffolded.contains("use terminal_testlib"));
    assert!(scaffolded.contains("test_daemon_terminal_processing"));
    assert!(scaffolded.contains("test_shared_memory_access"));
    assert!(scaffolded.contains("test_sixel_graphics_rendering"));

    // Step 7: Generate report
    let report = TestAuditor::generate_report(&placeholders);
    assert!(report.contains("Test Audit Report"));
    assert!(report.contains("**Placeholder Tests Found:** 3"));

    // Step 8: Generate summary
    let summary = TestAuditor::summarize(&placeholders);
    assert_eq!(summary.files_scanned, 1);
    assert_eq!(summary.placeholders_found, 3);

    Ok(())
}

#[test]
fn test_different_harness_types() {
    let test = terminal_testlib::audit::PlaceholderTest {
        file: std::path::PathBuf::from("test.rs"),
        line: 10,
        function_name: "test_async_operation".to_string(),
        inferred_subject: "async operation".to_string(),
        pattern: PlaceholderPattern::AssertTrue,
    };

    // Test TuiTestHarness
    let config = ScaffoldConfig {
        harness: HarnessType::TuiTestHarness,
        ..Default::default()
    };
    let template = TestAuditor::generate_template(&test, &config);
    assert!(template.contains("TuiTestHarness::new"));
    assert!(!template.contains("async fn"));

    // Test AsyncTuiTestHarness
    let config = ScaffoldConfig {
        harness: HarnessType::AsyncTuiTestHarness,
        ..Default::default()
    };
    let template = TestAuditor::generate_template(&test, &config);
    assert!(template.contains("AsyncTuiTestHarness::new"));
    assert!(template.contains("async fn"));

    // Test BevyTuiTestHarness
    let config = ScaffoldConfig {
        harness: HarnessType::BevyTuiTestHarness,
        ..Default::default()
    };
    let template = TestAuditor::generate_template(&test, &config);
    assert!(template.contains("BevyTuiTestHarness::new"));

    // Test ScarabTestHarness
    let config = ScaffoldConfig {
        harness: HarnessType::ScarabTestHarness,
        ..Default::default()
    };
    let template = TestAuditor::generate_template(&test, &config);
    assert!(template.contains("ScarabTestHarness::new"));
    assert!(template.contains("ScarabConfig"));
}

#[test]
fn test_recursive_directory_scanning() -> std::io::Result<()> {
    let temp_dir = TempDir::new()?;

    // Create nested directory structure
    let subdir1 = temp_dir.path().join("tests");
    let subdir2 = temp_dir.path().join("tests/integration");
    fs::create_dir_all(&subdir2)?;

    // Create test files at different levels
    fs::write(
        temp_dir.path().join("root_test.rs"),
        r#"
#[test]
fn test_root() {
    assert!(true);
}
"#,
    )?;

    fs::write(
        subdir1.join("level1_test.rs"),
        r#"
#[test]
fn test_level1() {
    todo!();
}
"#,
    )?;

    fs::write(
        subdir2.join("level2_test.rs"),
        r#"
#[test]
fn test_level2() {
    assert_eq!(0, 0);
}
"#,
    )?;

    // Scan recursively
    let placeholders = TestAuditor::find_placeholders_in_dir(temp_dir.path())?;

    // Verify all placeholders were found
    assert_eq!(placeholders.len(), 3);

    // Verify placeholders from different files
    assert!(placeholders.iter().any(|p| p.function_name == "test_root"));
    assert!(placeholders.iter().any(|p| p.function_name == "test_level1"));
    assert!(placeholders.iter().any(|p| p.function_name == "test_level2"));

    Ok(())
}

#[test]
fn test_subject_inference_keywords() {
    // Daemon/IPC keywords
    let test = terminal_testlib::audit::PlaceholderTest {
        file: std::path::PathBuf::from("test.rs"),
        line: 10,
        function_name: "test_daemon_ipc".to_string(),
        inferred_subject: "daemon ipc".to_string(),
        pattern: PlaceholderPattern::AssertTrue,
    };
    let config = ScaffoldConfig::default();
    let template = TestAuditor::generate_template(&test, &config);
    assert!(template.contains("daemon"));
    assert!(template.contains("control message") || template.contains("daemon state"));

    // Terminal/Screen keywords
    let test = terminal_testlib::audit::PlaceholderTest {
        file: std::path::PathBuf::from("test.rs"),
        line: 10,
        function_name: "test_terminal_output".to_string(),
        inferred_subject: "terminal output".to_string(),
        pattern: PlaceholderPattern::AssertTrue,
    };
    let template = TestAuditor::generate_template(&test, &config);
    assert!(template.contains("terminal") || template.contains("screen"));
    assert!(template.contains("wait_for") || template.contains("screen_contents"));

    // Sixel/Graphics keywords
    let test = terminal_testlib::audit::PlaceholderTest {
        file: std::path::PathBuf::from("test.rs"),
        line: 10,
        function_name: "test_sixel_rendering".to_string(),
        inferred_subject: "sixel rendering".to_string(),
        pattern: PlaceholderPattern::AssertTrue,
    };
    let template = TestAuditor::generate_template(&test, &config);
    assert!(template.contains("sixel") || template.contains("graphics"));
    assert!(template.contains("sixel_regions") || template.contains("graphics"));
}

#[test]
fn test_configuration_options() {
    let test = terminal_testlib::audit::PlaceholderTest {
        file: std::path::PathBuf::from("test.rs"),
        line: 10,
        function_name: "test_example".to_string(),
        inferred_subject: "example".to_string(),
        pattern: PlaceholderPattern::AssertTrue,
    };

    // Test with comments disabled
    let config = ScaffoldConfig {
        harness: HarnessType::TuiTestHarness,
        include_setup_teardown: false,
        generate_comments: false,
        include_error_handling: true,
    };
    let template = TestAuditor::generate_template(&test, &config);
    assert!(!template.contains("// Generated test template"));
    assert!(!template.contains("// Setup test harness"));

    // Test without error handling
    let config = ScaffoldConfig {
        harness: HarnessType::TuiTestHarness,
        include_setup_teardown: true,
        generate_comments: true,
        include_error_handling: false,
    };
    let template = TestAuditor::generate_template(&test, &config);
    assert!(!template.contains("-> Result<()>"));
    assert!(!template.contains("Ok(())"));

    // Test with all options enabled (default)
    let config = ScaffoldConfig::default();
    let template = TestAuditor::generate_template(&test, &config);
    assert!(template.contains("// Generated test template"));
    assert!(template.contains("-> Result<()>"));
    assert!(template.contains("Ok(())"));
}

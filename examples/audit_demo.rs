//! Demonstration of the test audit and scaffolding functionality.
//!
//! This example shows how to:
//! - Scan files for placeholder tests
//! - Generate audit reports
//! - Create test templates
//! - Scaffold complete test files

use terminal_testlib::audit::{
    HarnessType, PlaceholderPattern, PlaceholderTest, ScaffoldConfig, TestAuditor,
};
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

fn main() -> std::io::Result<()> {
    println!("=== Test Audit and Scaffolding Demo ===\n");

    // Create a temporary directory with sample test files
    let temp_dir = TempDir::new()?;
    create_sample_test_files(&temp_dir)?;

    // 1. Find placeholder tests
    println!("1. Scanning for placeholder tests...\n");
    let placeholders = TestAuditor::find_placeholders_in_dir(temp_dir.path())?;

    println!("Found {} placeholder tests\n", placeholders.len());

    // 2. Generate an audit report
    println!("2. Generating audit report...\n");
    let report = TestAuditor::generate_report(&placeholders);
    println!("{}\n", report);

    // 3. Generate summary statistics
    println!("3. Summary statistics...\n");
    let summary = TestAuditor::summarize(&placeholders);
    println!("Files scanned: {}", summary.files_scanned);
    println!("Placeholders found: {}", summary.placeholders_found);
    println!("By pattern:");
    for (pattern, count) in &summary.by_pattern {
        println!("  - {}: {}", pattern, count);
    }
    println!();

    // 4. Generate templates for different harness types
    println!("4. Generating test templates...\n");

    if let Some(test) = placeholders.first() {
        // Standard TUI harness
        println!("=== TuiTestHarness Template ===");
        let config = ScaffoldConfig {
            harness: HarnessType::TuiTestHarness,
            ..Default::default()
        };
        println!("{}\n", TestAuditor::generate_template(test, &config));

        // Async harness
        println!("=== AsyncTuiTestHarness Template ===");
        let config = ScaffoldConfig {
            harness: HarnessType::AsyncTuiTestHarness,
            ..Default::default()
        };
        println!("{}\n", TestAuditor::generate_template(test, &config));

        // Bevy harness
        println!("=== BevyTuiTestHarness Template ===");
        let config = ScaffoldConfig {
            harness: HarnessType::BevyTuiTestHarness,
            ..Default::default()
        };
        println!("{}\n", TestAuditor::generate_template(test, &config));

        // Scarab harness
        println!("=== ScarabTestHarness Template ===");
        let config = ScaffoldConfig {
            harness: HarnessType::ScarabTestHarness,
            ..Default::default()
        };
        println!("{}\n", TestAuditor::generate_template(test, &config));
    }

    // 5. Generate a complete scaffolded test file
    println!("5. Generating complete scaffolded test file...\n");
    let config = ScaffoldConfig::default();
    let scaffolded = TestAuditor::scaffold_test_file(&placeholders, &config);
    println!("{}\n", scaffolded);

    // 6. Demonstrate different placeholder patterns
    println!("6. Demonstrating different placeholder patterns...\n");
    demonstrate_patterns();

    Ok(())
}

fn create_sample_test_files(temp_dir: &TempDir) -> std::io::Result<()> {
    // Create first test file with various placeholders
    let test_file1 = temp_dir.path().join("daemon_tests.rs");
    fs::write(
        &test_file1,
        r#"
#[test]
fn test_daemon_terminal_processing() {
    assert!(true);
}

#[test]
fn test_daemon_ipc_communication() {
    todo!();
}

#[test]
fn test_shared_memory_access() {
    // TODO: implement this test
}

#[test]
fn test_terminal_input_handling() {
    assert_eq!(1, 1);
}

#[test]
fn test_sixel_graphics_rendering() {
}
"#,
    )?;

    // Create second test file
    let test_file2 = temp_dir.path().join("screen_tests.rs");
    fs::write(
        &test_file2,
        r#"
#[test]
fn test_screen_content_parsing() {
    assert!(true);
}

#[test]
fn test_bevy_ecs_integration() {
    unimplemented!();
}
"#,
    )?;

    Ok(())
}

fn demonstrate_patterns() {
    // Create example tests for each pattern type
    let examples = vec![
        PlaceholderTest {
            file: PathBuf::from("example.rs"),
            line: 10,
            function_name: "test_daemon_terminal_processing".to_string(),
            inferred_subject: "daemon terminal processing".to_string(),
            pattern: PlaceholderPattern::AssertTrue,
        },
        PlaceholderTest {
            file: PathBuf::from("example.rs"),
            line: 20,
            function_name: "test_shared_memory".to_string(),
            inferred_subject: "shared memory".to_string(),
            pattern: PlaceholderPattern::TrivialEquality,
        },
        PlaceholderTest {
            file: PathBuf::from("example.rs"),
            line: 30,
            function_name: "test_terminal_input".to_string(),
            inferred_subject: "terminal input".to_string(),
            pattern: PlaceholderPattern::TodoMacro,
        },
        PlaceholderTest {
            file: PathBuf::from("example.rs"),
            line: 40,
            function_name: "test_sixel_graphics".to_string(),
            inferred_subject: "sixel graphics".to_string(),
            pattern: PlaceholderPattern::EmptyBody,
        },
        PlaceholderTest {
            file: PathBuf::from("example.rs"),
            line: 50,
            function_name: "test_bevy_components".to_string(),
            inferred_subject: "bevy components".to_string(),
            pattern: PlaceholderPattern::CommentOnly,
        },
    ];

    println!("Pattern-specific templates:");
    let config = ScaffoldConfig::default();

    for test in examples {
        println!("\n--- Pattern: {} ---", test.pattern.as_str());
        println!("Subject: {}", test.inferred_subject);
        println!("\nGenerated template:");
        let template = TestAuditor::generate_template(&test, &config);
        // Just show first 10 lines to keep output manageable
        for (i, line) in template.lines().take(15).enumerate() {
            println!("{:3} | {}", i + 1, line);
        }
        if template.lines().count() > 15 {
            println!("    | ... ({} more lines)", template.lines().count() - 15);
        }
    }
}

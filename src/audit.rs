//! Test auditing and scaffolding utilities.
//!
//! This module provides tools for detecting placeholder tests and generating meaningful
//! test templates based on function names and test harness types.
//!
//! # Example
//!
//! ```rust,no_run
//! use terminal_testlib::audit::{TestAuditor, ScaffoldConfig, HarnessType};
//! use std::path::Path;
//!
//! # fn main() -> std::io::Result<()> {
//! // Find all placeholder tests in a directory
//! let placeholders = TestAuditor::find_placeholders_in_dir(Path::new("tests"))?;
//!
//! // Generate a report
//! let report = TestAuditor::generate_report(&placeholders);
//! println!("{}", report);
//!
//! // Generate templates for each placeholder
//! let config = ScaffoldConfig::default();
//! for test in &placeholders {
//!     let template = TestAuditor::generate_template(test, &config);
//!     println!("{}", template);
//! }
//! # Ok(())
//! # }
//! ```

use regex::Regex;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// A detected placeholder test.
#[derive(Debug, Clone)]
pub struct PlaceholderTest {
    /// File containing the test.
    pub file: PathBuf,
    /// Line number of the test function.
    pub line: usize,
    /// Name of the test function.
    pub function_name: String,
    /// Inferred test subject (parsed from function name).
    pub inferred_subject: String,
    /// The placeholder pattern detected.
    pub pattern: PlaceholderPattern,
}

/// Types of placeholder patterns.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PlaceholderPattern {
    /// `assert!(true)`
    AssertTrue,
    /// `assert_eq!(1, 1)` or similar trivial equality
    TrivialEquality,
    /// Empty test body
    EmptyBody,
    /// Only contains `todo!()` or `unimplemented!()`
    TodoMacro,
    /// Contains a comment like "// TODO" or "// placeholder"
    CommentOnly,
}

impl PlaceholderPattern {
    /// Get a human-readable string representation of the pattern.
    pub fn as_str(&self) -> &str {
        match self {
            PlaceholderPattern::AssertTrue => "assert!(true)",
            PlaceholderPattern::TrivialEquality => "Trivial equality",
            PlaceholderPattern::EmptyBody => "Empty body",
            PlaceholderPattern::TodoMacro => "todo!()/unimplemented!()",
            PlaceholderPattern::CommentOnly => "Comment only",
        }
    }
}

/// Type of test harness to generate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HarnessType {
    /// Standard TuiTestHarness
    TuiTestHarness,
    /// Scarab-specific harness
    ScarabTestHarness,
    /// Bevy TUI harness
    BevyTuiTestHarness,
    /// Async harness
    AsyncTuiTestHarness,
}

/// Configuration for test scaffolding.
#[derive(Debug, Clone)]
pub struct ScaffoldConfig {
    /// Type of harness to use.
    pub harness: HarnessType,
    /// Whether to include setup/teardown boilerplate.
    pub include_setup_teardown: bool,
    /// Whether to generate helpful comments.
    pub generate_comments: bool,
    /// Whether to include error handling.
    pub include_error_handling: bool,
}

impl Default for ScaffoldConfig {
    fn default() -> Self {
        Self {
            harness: HarnessType::TuiTestHarness,
            include_setup_teardown: true,
            generate_comments: true,
            include_error_handling: true,
        }
    }
}

/// Audit result summary.
#[derive(Debug, Clone, Default)]
pub struct AuditSummary {
    /// Total files scanned.
    pub files_scanned: usize,
    /// Total placeholder tests found.
    pub placeholders_found: usize,
    /// Breakdown by pattern type.
    pub by_pattern: HashMap<String, usize>,
    /// Files with placeholders.
    pub affected_files: Vec<PathBuf>,
}

/// Test auditor for finding and scaffolding tests.
#[derive(Debug)]
pub struct TestAuditor;

impl TestAuditor {
    /// Scan a file for placeholder tests.
    pub fn find_placeholders_in_file(path: &Path) -> std::io::Result<Vec<PlaceholderTest>> {
        let content = fs::read_to_string(path)?;
        let mut placeholders = Vec::new();

        // Regex to find test functions
        let test_fn_regex = Regex::new(r"#\[test\]\s*(?:async\s+)?fn\s+(\w+)\s*\(").unwrap();

        // Find all test functions
        for cap in test_fn_regex.captures_iter(&content) {
            let function_name = cap.get(1).unwrap().as_str().to_string();
            let start_pos = cap.get(0).unwrap().start();

            // Calculate line number
            let line = content[..start_pos].lines().count() + 1;

            // Extract the test body
            if let Some(body) = Self::extract_test_body(&content, start_pos) {
                if let Some(pattern) = Self::detect_placeholder_pattern(&body) {
                    let inferred_subject = Self::infer_subject(&function_name);

                    placeholders.push(PlaceholderTest {
                        file: path.to_path_buf(),
                        line,
                        function_name,
                        inferred_subject,
                        pattern,
                    });
                }
            }
        }

        Ok(placeholders)
    }

    /// Scan a directory recursively for placeholder tests.
    pub fn find_placeholders_in_dir(dir: &Path) -> std::io::Result<Vec<PlaceholderTest>> {
        let mut all_placeholders = Vec::new();

        if dir.is_file() {
            return Self::find_placeholders_in_file(dir);
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Recursively scan subdirectories
                all_placeholders.extend(Self::find_placeholders_in_dir(&path)?);
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                // Scan Rust files
                all_placeholders.extend(Self::find_placeholders_in_file(&path)?);
            }
        }

        Ok(all_placeholders)
    }

    /// Generate a replacement test template for a placeholder.
    pub fn generate_template(test: &PlaceholderTest, config: &ScaffoldConfig) -> String {
        let mut template = String::new();

        if config.generate_comments {
            template.push_str(&format!(
                "// Generated test template for: {}\n",
                test.function_name
            ));
            template.push_str(&format!("// Inferred subject: {}\n", test.inferred_subject));
            template.push_str(&format!(
                "// Original pattern: {}\n",
                test.pattern.as_str()
            ));
        }

        // Generate function signature
        let is_async = matches!(config.harness, HarnessType::AsyncTuiTestHarness);
        template.push_str("#[test]\n");

        if is_async {
            template.push_str(&format!("async fn {}()", test.function_name));
        } else {
            template.push_str(&format!("fn {}()", test.function_name));
        }

        if config.include_error_handling {
            template.push_str(" -> Result<()>");
        }

        template.push_str(" {\n");

        // Generate harness setup
        template.push_str(&Self::generate_harness_setup(config));

        // Generate test body based on inferred subject
        template.push_str(&Self::generate_test_body(&test.inferred_subject, config));

        // Generate teardown if needed
        if config.include_setup_teardown {
            template.push_str(&Self::generate_teardown(config));
        }

        if config.include_error_handling {
            template.push_str("    Ok(())\n");
        }

        template.push_str("}\n");

        template
    }

    /// Generate a full test file with meaningful tests.
    pub fn scaffold_test_file(tests: &[PlaceholderTest], config: &ScaffoldConfig) -> String {
        let mut output = String::new();

        // Add file header
        output.push_str("// Scaffolded test file\n");
        output.push_str("// Generated by terminal_testlib::audit::TestAuditor\n\n");

        // Add imports based on harness type
        output.push_str(&Self::generate_imports(config));
        output.push_str("\n");

        // Generate each test
        for (i, test) in tests.iter().enumerate() {
            if i > 0 {
                output.push_str("\n");
            }
            output.push_str(&Self::generate_template(test, config));
        }

        output
    }

    /// Generate an audit report as markdown.
    pub fn generate_report(tests: &[PlaceholderTest]) -> String {
        let summary = Self::summarize(tests);
        let mut report = String::new();

        report.push_str("# Test Audit Report\n\n");
        report.push_str(&format!("**Files Scanned:** {}\n", summary.files_scanned));
        report.push_str(&format!(
            "**Placeholder Tests Found:** {}\n\n",
            summary.placeholders_found
        ));

        if summary.placeholders_found > 0 {
            report.push_str("## Breakdown by Pattern\n\n");
            let mut patterns: Vec<_> = summary.by_pattern.iter().collect();
            patterns.sort_by_key(|(_, count)| std::cmp::Reverse(*count));

            for (pattern, count) in patterns {
                report.push_str(&format!("- **{}**: {} tests\n", pattern, count));
            }

            report.push_str("\n## Affected Files\n\n");
            for file in &summary.affected_files {
                let file_tests: Vec<_> = tests
                    .iter()
                    .filter(|t| t.file == *file)
                    .collect();

                report.push_str(&format!(
                    "### {} ({} placeholders)\n\n",
                    file.display(),
                    file_tests.len()
                ));

                for test in file_tests {
                    report.push_str(&format!(
                        "- Line {}: `{}` - {} (subject: {})\n",
                        test.line,
                        test.function_name,
                        test.pattern.as_str(),
                        test.inferred_subject
                    ));
                }

                report.push_str("\n");
            }
        }

        report
    }

    /// Generate summary statistics.
    pub fn summarize(tests: &[PlaceholderTest]) -> AuditSummary {
        let mut by_pattern: HashMap<String, usize> = HashMap::new();
        let mut affected_files: Vec<PathBuf> = Vec::new();

        for test in tests {
            *by_pattern.entry(test.pattern.as_str().to_string()).or_insert(0) += 1;

            if !affected_files.contains(&test.file) {
                affected_files.push(test.file.clone());
            }
        }

        affected_files.sort();

        AuditSummary {
            files_scanned: affected_files.len(),
            placeholders_found: tests.len(),
            by_pattern,
            affected_files,
        }
    }

    // Helper methods

    fn extract_test_body(content: &str, start_pos: usize) -> Option<String> {
        let remaining = &content[start_pos..];

        // Find the opening brace
        let open_brace = remaining.find('{')?;
        let mut depth = 0;
        let mut end_pos = None;

        // Find the matching closing brace
        for (i, ch) in remaining[open_brace..].char_indices() {
            match ch {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end_pos = Some(open_brace + i);
                        break;
                    }
                }
                _ => {}
            }
        }

        let end = end_pos?;
        Some(remaining[open_brace + 1..end].to_string())
    }

    fn detect_placeholder_pattern(body: &str) -> Option<PlaceholderPattern> {
        let trimmed = body.trim();

        // Empty body
        if trimmed.is_empty() {
            return Some(PlaceholderPattern::EmptyBody);
        }

        // Remove comments and whitespace for analysis
        let code_only: String = trimmed
            .lines()
            .filter(|line| !line.trim().starts_with("//"))
            .collect::<Vec<_>>()
            .join("\n")
            .trim()
            .to_string();

        // Check for todo/unimplemented
        if code_only.contains("todo!()") || code_only.contains("unimplemented!()") {
            return Some(PlaceholderPattern::TodoMacro);
        }

        // Check for assert!(true)
        if code_only.contains("assert!(true)") || code_only.contains("assert!( true )") {
            return Some(PlaceholderPattern::AssertTrue);
        }

        // Check for trivial equality
        let trivial_patterns = [
            r"assert_eq!\s*\(\s*1\s*,\s*1\s*\)",
            r"assert_eq!\s*\(\s*0\s*,\s*0\s*\)",
            r"assert_eq!\s*\(\s*true\s*,\s*true\s*\)",
            r"assert_eq!\s*\(\s*false\s*,\s*false\s*\)",
        ];

        for pattern in &trivial_patterns {
            if Regex::new(pattern).unwrap().is_match(&code_only) {
                return Some(PlaceholderPattern::TrivialEquality);
            }
        }

        // Check for comment-only body
        if code_only.is_empty() && !trimmed.is_empty() {
            return Some(PlaceholderPattern::CommentOnly);
        }

        None
    }

    fn infer_subject(function_name: &str) -> String {
        // Remove "test_" prefix if present
        let name = function_name.strip_prefix("test_").unwrap_or(function_name);

        // Split by underscores and create a readable subject
        let parts: Vec<&str> = name.split('_').collect();

        if parts.is_empty() {
            return "unknown".to_string();
        }

        // Try to identify key components
        let subject = parts.join(" ");
        subject
    }

    fn generate_imports(config: &ScaffoldConfig) -> String {
        match config.harness {
            HarnessType::TuiTestHarness => {
                "use terminal_testlib::{TuiTestHarness, Result};\n\
                 use portable_pty::CommandBuilder;"
                    .to_string()
            }
            HarnessType::AsyncTuiTestHarness => {
                "use terminal_testlib::{AsyncTuiTestHarness, Result};\n\
                 use portable_pty::CommandBuilder;\n\
                 use tokio::test;"
                    .to_string()
            }
            HarnessType::BevyTuiTestHarness => {
                "use terminal_testlib::{BevyTuiTestHarness, Result};"
                    .to_string()
            }
            HarnessType::ScarabTestHarness => {
                "use terminal_testlib::{ScarabTestHarness, ScarabConfig, Result};"
                    .to_string()
            }
        }
    }

    fn generate_harness_setup(config: &ScaffoldConfig) -> String {
        let mut setup = String::new();

        if config.generate_comments {
            setup.push_str("    // Setup test harness\n");
        }

        match config.harness {
            HarnessType::TuiTestHarness => {
                setup.push_str("    let mut harness = TuiTestHarness::new(80, 24)?;\n");
                if config.generate_comments {
                    setup.push_str("    // Spawn your TUI application\n");
                    setup.push_str("    // let mut cmd = CommandBuilder::new(\"./your-app\");\n");
                    setup.push_str("    // harness.spawn(cmd)?;\n");
                }
            }
            HarnessType::AsyncTuiTestHarness => {
                setup.push_str("    let mut harness = AsyncTuiTestHarness::new(80, 24).await?;\n");
                if config.generate_comments {
                    setup.push_str("    // Spawn your async TUI application\n");
                    setup.push_str("    // let mut cmd = CommandBuilder::new(\"./your-app\");\n");
                    setup.push_str("    // harness.spawn(cmd).await?;\n");
                }
            }
            HarnessType::BevyTuiTestHarness => {
                setup.push_str("    let mut harness = BevyTuiTestHarness::new()?;\n");
                if config.generate_comments {
                    setup.push_str("    // Add your Bevy systems and components\n");
                    setup.push_str("    // harness.app.add_systems(Update, your_system);\n");
                }
            }
            HarnessType::ScarabTestHarness => {
                setup.push_str("    let config = ScarabConfig::builder()\n");
                setup.push_str("        .daemon_path(\"./your-daemon\")\n");
                setup.push_str("        .build();\n");
                setup.push_str("    let mut harness = ScarabTestHarness::new(config)?;\n");
            }
        }

        setup.push_str("\n");
        setup
    }

    fn generate_test_body(subject: &str, config: &ScaffoldConfig) -> String {
        let mut body = String::new();

        if config.generate_comments {
            body.push_str(&format!("    // Test: {}\n", subject));
        }

        // Generate test body based on keywords in subject
        if subject.contains("daemon") || subject.contains("ipc") {
            body.push_str("    // TODO: Send control messages to daemon\n");
            body.push_str("    // harness.send_control_message(ControlMessage::YourMessage)?;\n");
            body.push_str("\n");
            body.push_str("    // TODO: Verify daemon state\n");
            body.push_str("    // let state = harness.read_shared_state()?;\n");
            body.push_str("    // assert_eq!(state.field, expected_value);\n");
        } else if subject.contains("terminal") || subject.contains("screen") {
            body.push_str("    // TODO: Wait for expected screen content\n");
            body.push_str("    // harness.wait_for(|state| state.contents().contains(\"expected\"))?;\n");
            body.push_str("\n");
            body.push_str("    // TODO: Verify screen state\n");
            body.push_str("    // let contents = harness.screen_contents();\n");
            body.push_str("    // assert!(contents.contains(\"expected text\"));\n");
        } else if subject.contains("input") || subject.contains("key") {
            body.push_str("    // TODO: Send keyboard input\n");
            body.push_str("    // harness.send_text(\"test input\")?;\n");
            body.push_str("\n");
            body.push_str("    // TODO: Verify response to input\n");
            body.push_str("    // harness.wait_for(|state| state.contents().contains(\"test input\"))?;\n");
        } else if subject.contains("shared") || subject.contains("memory") {
            body.push_str("    // TODO: Access shared memory state\n");
            body.push_str("    // let state = harness.read_shared_state()?;\n");
            body.push_str("\n");
            body.push_str("    // TODO: Verify shared state contents\n");
            body.push_str("    // assert_eq!(state.field, expected_value);\n");
        } else if subject.contains("sixel") || subject.contains("graphics") {
            body.push_str("    // TODO: Trigger graphics rendering\n");
            body.push_str("    // harness.send_text(\"trigger_render\")?;\n");
            body.push_str("\n");
            body.push_str("    // TODO: Verify Sixel regions\n");
            body.push_str("    // let sixel_regions = harness.state().sixel_regions();\n");
            body.push_str("    // assert!(!sixel_regions.is_empty());\n");
        } else if subject.contains("bevy") || subject.contains("ecs") {
            body.push_str("    // TODO: Update Bevy world\n");
            body.push_str("    // harness.app.update();\n");
            body.push_str("\n");
            body.push_str("    // TODO: Query and verify components\n");
            body.push_str("    // let world = &harness.app.world;\n");
            body.push_str("    // let query_result = world.query::<&YourComponent>();\n");
            body.push_str("    // assert!(query_result.iter(world).count() > 0);\n");
        } else {
            body.push_str("    // TODO: Implement test logic\n");
            body.push_str("    // Add your test assertions here\n");
        }

        body.push_str("\n");
        body
    }

    fn generate_teardown(config: &ScaffoldConfig) -> String {
        let mut teardown = String::new();

        if config.generate_comments {
            teardown.push_str("    // Cleanup (automatic with Drop trait)\n");
        }

        teardown
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_detect_assert_true_pattern() {
        let body = "assert!(true);";
        let pattern = TestAuditor::detect_placeholder_pattern(body);
        assert_eq!(pattern, Some(PlaceholderPattern::AssertTrue));
    }

    #[test]
    fn test_detect_trivial_equality_pattern() {
        let body = "assert_eq!(1, 1);";
        let pattern = TestAuditor::detect_placeholder_pattern(body);
        assert_eq!(pattern, Some(PlaceholderPattern::TrivialEquality));
    }

    #[test]
    fn test_detect_empty_body_pattern() {
        let body = "";
        let pattern = TestAuditor::detect_placeholder_pattern(body);
        assert_eq!(pattern, Some(PlaceholderPattern::EmptyBody));
    }

    #[test]
    fn test_detect_todo_macro_pattern() {
        let body = "todo!();";
        let pattern = TestAuditor::detect_placeholder_pattern(body);
        assert_eq!(pattern, Some(PlaceholderPattern::TodoMacro));
    }

    #[test]
    fn test_detect_comment_only_pattern() {
        let body = "// TODO: implement this test";
        let pattern = TestAuditor::detect_placeholder_pattern(body);
        assert_eq!(pattern, Some(PlaceholderPattern::CommentOnly));
    }

    #[test]
    fn test_infer_subject() {
        assert_eq!(
            TestAuditor::infer_subject("test_daemon_terminal_processing"),
            "daemon terminal processing"
        );
        assert_eq!(
            TestAuditor::infer_subject("test_shared_memory"),
            "shared memory"
        );
        assert_eq!(
            TestAuditor::infer_subject("daemon_ipc_communication"),
            "daemon ipc communication"
        );
    }

    #[test]
    fn test_find_placeholders_in_file() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;
        let test_file = temp_dir.path().join("test.rs");

        let content = r#"
#[test]
fn test_placeholder_assert_true() {
    assert!(true);
}

#[test]
fn test_placeholder_trivial_eq() {
    assert_eq!(1, 1);
}

#[test]
fn test_real_test() {
    let x = compute_value();
    assert_eq!(x, 42);
}

#[test]
fn test_empty() {
}
"#;

        fs::write(&test_file, content)?;

        let placeholders = TestAuditor::find_placeholders_in_file(&test_file)?;

        assert_eq!(placeholders.len(), 3);
        assert_eq!(placeholders[0].function_name, "test_placeholder_assert_true");
        assert_eq!(placeholders[0].pattern, PlaceholderPattern::AssertTrue);
        assert_eq!(placeholders[1].function_name, "test_placeholder_trivial_eq");
        assert_eq!(placeholders[1].pattern, PlaceholderPattern::TrivialEquality);
        assert_eq!(placeholders[2].function_name, "test_empty");
        assert_eq!(placeholders[2].pattern, PlaceholderPattern::EmptyBody);

        Ok(())
    }

    #[test]
    fn test_generate_template_basic() {
        let test = PlaceholderTest {
            file: PathBuf::from("test.rs"),
            line: 10,
            function_name: "test_daemon_processing".to_string(),
            inferred_subject: "daemon processing".to_string(),
            pattern: PlaceholderPattern::AssertTrue,
        };

        let config = ScaffoldConfig::default();
        let template = TestAuditor::generate_template(&test, &config);

        assert!(template.contains("test_daemon_processing"));
        assert!(template.contains("TuiTestHarness"));
        assert!(template.contains("daemon"));
    }

    #[test]
    fn test_generate_template_async() {
        let test = PlaceholderTest {
            file: PathBuf::from("test.rs"),
            line: 10,
            function_name: "test_async_operation".to_string(),
            inferred_subject: "async operation".to_string(),
            pattern: PlaceholderPattern::TodoMacro,
        };

        let config = ScaffoldConfig {
            harness: HarnessType::AsyncTuiTestHarness,
            ..Default::default()
        };
        let template = TestAuditor::generate_template(&test, &config);

        assert!(template.contains("async fn"));
        assert!(template.contains("AsyncTuiTestHarness"));
    }

    #[test]
    fn test_generate_report() {
        let placeholders = vec![
            PlaceholderTest {
                file: PathBuf::from("test1.rs"),
                line: 10,
                function_name: "test_a".to_string(),
                inferred_subject: "a".to_string(),
                pattern: PlaceholderPattern::AssertTrue,
            },
            PlaceholderTest {
                file: PathBuf::from("test1.rs"),
                line: 20,
                function_name: "test_b".to_string(),
                inferred_subject: "b".to_string(),
                pattern: PlaceholderPattern::AssertTrue,
            },
            PlaceholderTest {
                file: PathBuf::from("test2.rs"),
                line: 5,
                function_name: "test_c".to_string(),
                inferred_subject: "c".to_string(),
                pattern: PlaceholderPattern::TrivialEquality,
            },
        ];

        let report = TestAuditor::generate_report(&placeholders);

        assert!(report.contains("Test Audit Report"));
        assert!(report.contains("**Files Scanned:** 2"));
        assert!(report.contains("**Placeholder Tests Found:** 3"));
        assert!(report.contains("**assert!(true)**: 2 tests"));
        assert!(report.contains("**Trivial equality**: 1 tests"));
    }

    #[test]
    fn test_summarize() {
        let placeholders = vec![
            PlaceholderTest {
                file: PathBuf::from("test1.rs"),
                line: 10,
                function_name: "test_a".to_string(),
                inferred_subject: "a".to_string(),
                pattern: PlaceholderPattern::AssertTrue,
            },
            PlaceholderTest {
                file: PathBuf::from("test1.rs"),
                line: 20,
                function_name: "test_b".to_string(),
                inferred_subject: "b".to_string(),
                pattern: PlaceholderPattern::AssertTrue,
            },
        ];

        let summary = TestAuditor::summarize(&placeholders);

        assert_eq!(summary.files_scanned, 1);
        assert_eq!(summary.placeholders_found, 2);
        assert_eq!(summary.by_pattern.get("assert!(true)"), Some(&2));
        assert_eq!(summary.affected_files.len(), 1);
    }

    #[test]
    fn test_scaffold_test_file() {
        let placeholders = vec![
            PlaceholderTest {
                file: PathBuf::from("test.rs"),
                line: 10,
                function_name: "test_daemon_processing".to_string(),
                inferred_subject: "daemon processing".to_string(),
                pattern: PlaceholderPattern::AssertTrue,
            },
            PlaceholderTest {
                file: PathBuf::from("test.rs"),
                line: 20,
                function_name: "test_terminal_output".to_string(),
                inferred_subject: "terminal output".to_string(),
                pattern: PlaceholderPattern::EmptyBody,
            },
        ];

        let config = ScaffoldConfig::default();
        let scaffolded = TestAuditor::scaffold_test_file(&placeholders, &config);

        assert!(scaffolded.contains("Scaffolded test file"));
        assert!(scaffolded.contains("test_daemon_processing"));
        assert!(scaffolded.contains("test_terminal_output"));
        assert!(scaffolded.contains("use terminal_testlib"));
    }

    #[test]
    fn test_extract_test_body() {
        let content = r#"
#[test]
fn test_example() {
    let x = 1;
    assert_eq!(x, 1);
}
"#;

        let start_pos = content.find("#[test]").unwrap();
        let body = TestAuditor::extract_test_body(content, start_pos).unwrap();

        assert!(body.contains("let x = 1"));
        assert!(body.contains("assert_eq!(x, 1)"));
        assert!(!body.contains("fn test_example"));
    }

    #[test]
    fn test_nested_braces_in_test_body() {
        let content = r#"
#[test]
fn test_with_nested() {
    if true {
        let x = { 1 + 1 };
        assert_eq!(x, 2);
    }
}
"#;

        let start_pos = content.find("#[test]").unwrap();
        let body = TestAuditor::extract_test_body(content, start_pos).unwrap();

        assert!(body.contains("if true"));
        assert!(body.contains("let x = { 1 + 1 }"));
    }

    #[test]
    fn test_find_placeholders_in_dir() -> std::io::Result<()> {
        let temp_dir = TempDir::new()?;

        // Create test file 1
        let test_file1 = temp_dir.path().join("test1.rs");
        fs::write(&test_file1, r#"
#[test]
fn test_placeholder() {
    assert!(true);
}
"#)?;

        // Create subdirectory with test file 2
        let sub_dir = temp_dir.path().join("subdir");
        fs::create_dir(&sub_dir)?;
        let test_file2 = sub_dir.join("test2.rs");
        fs::write(&test_file2, r#"
#[test]
fn test_another_placeholder() {
    todo!();
}
"#)?;

        let placeholders = TestAuditor::find_placeholders_in_dir(temp_dir.path())?;

        assert_eq!(placeholders.len(), 2);

        Ok(())
    }
}

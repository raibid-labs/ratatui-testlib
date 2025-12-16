# Test Audit and Scaffolding Guide

The `audit` module in `terminal-testlib` provides utilities for detecting placeholder tests and generating meaningful test templates. This is particularly useful for:

- Identifying incomplete or placeholder tests in your codebase
- Generating meaningful test stubs based on function names
- Auditing test coverage and quality
- Scaffolding tests for different harness types

## Quick Start

```rust
use terminal_testlib::audit::{TestAuditor, ScaffoldConfig, HarnessType};
use std::path::Path;

// Find all placeholder tests in a directory
let placeholders = TestAuditor::find_placeholders_in_dir(Path::new("tests"))?;

// Generate a markdown audit report
let report = TestAuditor::generate_report(&placeholders);
println!("{}", report);

// Generate test templates
let config = ScaffoldConfig::default();
for test in &placeholders {
    let template = TestAuditor::generate_template(test, &config);
    println!("{}", template);
}
```

## Placeholder Patterns Detected

The auditor can detect several types of placeholder tests:

### 1. `assert!(true)`

```rust
#[test]
fn test_something() {
    assert!(true);
}
```

### 2. Trivial Equality

```rust
#[test]
fn test_something() {
    assert_eq!(1, 1);
    assert_eq!(0, 0);
    assert_eq!(true, true);
}
```

### 3. Empty Body

```rust
#[test]
fn test_something() {
}
```

### 4. Todo Macros

```rust
#[test]
fn test_something() {
    todo!();
}

#[test]
fn test_something_else() {
    unimplemented!();
}
```

### 5. Comment Only

```rust
#[test]
fn test_something() {
    // TODO: implement this test
}
```

## Test Template Generation

The auditor can generate intelligent test templates based on function names. It parses the function name to infer what should be tested and generates appropriate boilerplate.

### Example: Daemon Test

Function: `test_daemon_terminal_processing`

Generated template:
```rust
// Generated test template for: test_daemon_terminal_processing
// Inferred subject: daemon terminal processing
// Original pattern: assert!(true)
#[test]
fn test_daemon_terminal_processing() -> Result<()> {
    // Setup test harness
    let mut harness = TuiTestHarness::new(80, 24)?;
    // Spawn your TUI application
    // let mut cmd = CommandBuilder::new("./your-app");
    // harness.spawn(cmd)?;

    // Test: daemon terminal processing
    // TODO: Send control messages to daemon
    // harness.send_control_message(ControlMessage::YourMessage)?;

    // TODO: Verify daemon state
    // let state = harness.read_shared_state()?;
    // assert_eq!(state.field, expected_value);

    // Cleanup (automatic with Drop trait)
    Ok(())
}
```

### Subject Inference

The auditor infers test subjects from function names:

- `test_daemon_terminal_processing` → "daemon terminal processing"
- `test_shared_memory` → "shared memory"
- `test_sixel_graphics_rendering` → "sixel graphics rendering"
- `test_bevy_ecs_integration` → "bevy ecs integration"

Based on keywords in the subject, it generates appropriate test boilerplate:

| Keywords | Generated Boilerplate |
|----------|----------------------|
| daemon, ipc | Control message sending, shared state verification |
| terminal, screen | Screen content waiting, text verification |
| input, key | Keyboard input sending, response verification |
| shared, memory | Shared state access and verification |
| sixel, graphics | Graphics rendering, Sixel region verification |
| bevy, ecs | Bevy world updates, component queries |

## Harness Types

The auditor supports generating templates for different test harness types:

### 1. TuiTestHarness (Default)

Standard PTY-based test harness:

```rust
let config = ScaffoldConfig {
    harness: HarnessType::TuiTestHarness,
    ..Default::default()
};
```

### 2. AsyncTuiTestHarness

Async/await support with Tokio:

```rust
let config = ScaffoldConfig {
    harness: HarnessType::AsyncTuiTestHarness,
    ..Default::default()
};
```

Generates:
```rust
#[test]
async fn test_async_operation() -> Result<()> {
    let mut harness = AsyncTuiTestHarness::new(80, 24).await?;
    // ...
}
```

### 3. BevyTuiTestHarness

Bevy ECS integration:

```rust
let config = ScaffoldConfig {
    harness: HarnessType::BevyTuiTestHarness,
    ..Default::default()
};
```

Generates:
```rust
#[test]
fn test_bevy_components() -> Result<()> {
    let mut harness = BevyTuiTestHarness::new()?;
    // Add your Bevy systems and components
    // harness.app.add_systems(Update, your_system);
    // ...
}
```

### 4. ScarabTestHarness

Scarab-specific IPC testing:

```rust
let config = ScaffoldConfig {
    harness: HarnessType::ScarabTestHarness,
    ..Default::default()
};
```

Generates:
```rust
#[test]
fn test_scarab_daemon() -> Result<()> {
    let config = ScarabConfig::builder()
        .daemon_path("./your-daemon")
        .build();
    let mut harness = ScarabTestHarness::new(config)?;
    // ...
}
```

## Configuration Options

`ScaffoldConfig` provides several options for customizing generated templates:

```rust
let config = ScaffoldConfig {
    harness: HarnessType::TuiTestHarness,
    include_setup_teardown: true,    // Include setup/teardown comments
    generate_comments: true,          // Generate helpful comments
    include_error_handling: true,     // Add Result<()> return type
};
```

## Audit Reports

Generate markdown reports for documentation or code review:

```rust
let report = TestAuditor::generate_report(&placeholders);
```

Example output:

```markdown
# Test Audit Report

**Files Scanned:** 2
**Placeholder Tests Found:** 5

## Breakdown by Pattern

- **assert!(true)**: 2 tests
- **todo!()/unimplemented!()**: 2 tests
- **Empty body**: 1 tests

## Affected Files

### tests/daemon_tests.rs (3 placeholders)

- Line 10: `test_daemon_processing` - assert!(true) (subject: daemon processing)
- Line 20: `test_ipc_communication` - todo!()/unimplemented!() (subject: ipc communication)
- Line 30: `test_shared_memory` - Empty body (subject: shared memory)
```

## CLI Tool

Use the provided CLI tool for quick audits:

```bash
# Scan for placeholders
cargo run --example audit_cli -- scan tests/

# Generate report
cargo run --example audit_cli -- report tests/ > audit-report.md

# Show summary
cargo run --example audit_cli -- summary tests/

# Generate scaffolded tests
cargo run --example audit_cli -- scaffold tests/my_test.rs --harness async
```

## Integration with CI/CD

You can integrate the auditor into your CI pipeline:

```rust
use terminal_testlib::audit::TestAuditor;
use std::path::Path;

#[test]
fn test_no_placeholder_tests() {
    let placeholders = TestAuditor::find_placeholders_in_dir(Path::new("tests"))
        .expect("Failed to scan tests");

    if !placeholders.is_empty() {
        let report = TestAuditor::generate_report(&placeholders);
        panic!("Found {} placeholder tests:\n{}", placeholders.len(), report);
    }
}
```

## Best Practices

1. **Regular Audits**: Run the auditor regularly to catch placeholder tests early
2. **Incremental Replacement**: Replace placeholders incrementally to maintain test quality
3. **Use Subject Inference**: Structure test function names to leverage the subject inference
4. **Custom Configuration**: Adjust `ScaffoldConfig` for your project's testing patterns
5. **CI Integration**: Add automated checks to prevent placeholder tests from being merged

## API Reference

### TestAuditor

Static methods for test auditing:

- `find_placeholders_in_file(path)` - Scan a single file
- `find_placeholders_in_dir(dir)` - Scan a directory recursively
- `generate_template(test, config)` - Generate a test template
- `scaffold_test_file(tests, config)` - Generate a complete test file
- `generate_report(tests)` - Generate markdown audit report
- `summarize(tests)` - Generate summary statistics

### PlaceholderTest

Represents a detected placeholder test:

```rust
pub struct PlaceholderTest {
    pub file: PathBuf,
    pub line: usize,
    pub function_name: String,
    pub inferred_subject: String,
    pub pattern: PlaceholderPattern,
}
```

### PlaceholderPattern

Types of placeholder patterns:

- `AssertTrue`
- `TrivialEquality`
- `EmptyBody`
- `TodoMacro`
- `CommentOnly`

### ScaffoldConfig

Configuration for template generation:

```rust
pub struct ScaffoldConfig {
    pub harness: HarnessType,
    pub include_setup_teardown: bool,
    pub generate_comments: bool,
    pub include_error_handling: bool,
}
```

### AuditSummary

Summary statistics:

```rust
pub struct AuditSummary {
    pub files_scanned: usize,
    pub placeholders_found: usize,
    pub by_pattern: HashMap<String, usize>,
    pub affected_files: Vec<PathBuf>,
}
```

# Scarab - File Manager Testing Patterns

> Testing patterns for file manager and file browser TUIs

## Overview

Scarab is a TUI file manager with features like directory navigation, file operations, and dual-pane browsing. This cookbook demonstrates how to test file manager applications using terminal-testlib.

**Target Audience**: Developers building file browsers, file managers, or directory navigation tools.

## Setup

### Test Harness Configuration

```rust
use terminal_testlib::{TuiTestHarness, KeyCode};
use std::path::PathBuf;

fn create_file_manager_harness() -> terminal_testlib::Result<TuiTestHarness> {
    let mut harness = TuiTestHarness::new(120, 40)?; // Wider for dual-pane
    harness.set_timeout(Duration::from_secs(2));
    Ok(harness)
}
```

### Test Data Setup

```rust
use tempfile::TempDir;
use std::fs;

fn create_test_directory() -> TempDir {
    let temp = TempDir::new().unwrap();
    let base = temp.path();

    // Create test directory structure
    fs::create_dir_all(base.join("folder1/subfolder")).unwrap();
    fs::create_dir_all(base.join("folder2")).unwrap();
    fs::write(base.join("file1.txt"), "content1").unwrap();
    fs::write(base.join("file2.md"), "content2").unwrap();
    fs::write(base.join("folder1/nested.txt"), "nested").unwrap();

    temp
}
```

## Common Patterns

### Pattern 1: File List Navigation

**Use Case**: Testing cursor movement through file lists

```rust
#[test]
fn test_file_list_navigation() -> Result<()> {
    let temp_dir = create_test_directory();
    let mut harness = create_file_manager_harness()?;

    // Spawn file manager with test directory
    let mut cmd = CommandBuilder::new("scarab");
    cmd.arg(temp_dir.path());
    harness.spawn(cmd)?;

    // Wait for initial render
    harness.wait_for_text("file1.txt")?;

    // Move down through files
    harness.send_key(KeyCode::Down)?;
    harness.send_key(KeyCode::Down)?;

    // Verify selection indicator moved
    harness.assert_text_at(2, 0, "►")?; // Selection arrow

    // Verify highlighted filename
    harness.assert_highlighted("folder1")?;

    Ok(())
}
```

**Expected Behavior**:
- Cursor moves down on Down key
- Selection indicator follows cursor
- Currently selected item is highlighted

**Common Pitfalls**:
- Not waiting for directory scan to complete
- Hard-coding line numbers (files may sort differently)
- Assuming specific terminal colors

---

### Pattern 2: Directory Tree Navigation

**Use Case**: Testing entering and exiting directories

```rust
#[test]
fn test_directory_navigation() -> Result<()> {
    let temp_dir = create_test_directory();
    let mut harness = create_file_manager_harness()?;

    let mut cmd = CommandBuilder::new("scarab");
    cmd.arg(temp_dir.path());
    harness.spawn(cmd)?;

    harness.wait_for_text("folder1")?;

    // Navigate to folder1
    harness.send_key(KeyCode::Down)?; // Move to folder1
    harness.send_key(KeyCode::Enter)?; // Enter directory

    // Verify we're in the subdirectory
    harness.wait_for_text("subfolder")?;
    harness.wait_for_text("nested.txt")?;

    // Navigate back up
    harness.send_key(KeyCode::Char('h'))?; // Vim-style back
    // Or: harness.send_key(KeyCode::Backspace)?;

    // Verify we're back at parent
    harness.wait_for_text("file1.txt")?;
    harness.wait_for_text("folder1")?;

    Ok(())
}
```

---

### Pattern 3: Dual-Pane Layout Testing

**Use Case**: Testing split-screen file browsing

```rust
#[test]
fn test_dual_pane_layout() -> Result<()> {
    let temp_dir = create_test_directory();
    let mut harness = create_file_manager_harness()?;

    let mut cmd = CommandBuilder::new("scarab");
    cmd.arg("--dual-pane");
    harness.spawn(cmd)?;

    harness.wait_for_stable()?;

    // Verify left pane has content
    let left_area = Rect::new(0, 1, 60, 38);
    harness.assert_text_in_area(left_area, "file1.txt")?;

    // Verify right pane has content
    let right_area = Rect::new(60, 1, 60, 38);
    harness.assert_text_in_area(right_area, "file1.txt")?;

    // Switch focus to right pane
    harness.send_key(KeyCode::Tab)?;

    // Verify focus indicator moved
    harness.assert_text_in_area(right_area, "►")?;

    Ok(())
}
```

---

### Pattern 4: File Operation Confirmations

**Use Case**: Testing delete/copy/move confirmation dialogs

```rust
#[test]
fn test_delete_confirmation() -> Result<()> {
    let temp_dir = create_test_directory();
    let mut harness = create_file_manager_harness()?;

    let mut cmd = CommandBuilder::new("scarab");
    cmd.arg(temp_dir.path());
    harness.spawn(cmd)?;

    harness.wait_for_text("file1.txt")?;

    // Select file and trigger delete
    harness.send_key(KeyCode::Down)?;
    harness.send_key(KeyCode::Char('d'))?; // Delete key

    // Verify confirmation popup appears
    harness.wait_for_text("Delete file1.txt?")?;
    harness.assert_text_contains("Are you sure")?;

    // Confirm deletion
    harness.send_key(KeyCode::Char('y'))?;

    // Verify file is gone
    harness.wait_until(|| {
        !harness.screen_text().contains("file1.txt")
    }, Duration::from_secs(1))?;

    // Verify file actually deleted from filesystem
    assert!(!temp_dir.path().join("file1.txt").exists());

    Ok(())
}
```

---

### Pattern 5: Search/Filter Functionality

**Use Case**: Testing file filtering and search

```rust
#[test]
fn test_file_search() -> Result<()> {
    let temp_dir = create_test_directory();
    let mut harness = create_file_manager_harness()?;

    let mut cmd = CommandBuilder::new("scarab");
    cmd.arg(temp_dir.path());
    harness.spawn(cmd)?;

    harness.wait_for_stable()?;

    // Activate search mode
    harness.send_key(KeyCode::Char('/'))?;

    // Type search query
    harness.type_text("txt")?;

    // Verify only .txt files shown
    harness.wait_for_text("file1.txt")?;
    assert!(!harness.screen_text().contains("file2.md"));

    // Clear search
    harness.send_key(KeyCode::Esc)?;

    // Verify all files shown again
    harness.wait_for_text("file2.md")?;

    Ok(())
}
```

---

## Real-World Examples

### Complete Integration Test

```rust
use terminal_testlib::{TuiTestHarness, KeyCode};
use portable_pty::CommandBuilder;
use tempfile::TempDir;
use std::fs;

#[test]
fn test_full_file_manager_workflow() -> terminal_testlib::Result<()> {
    // Setup test environment
    let temp_dir = TempDir::new()?;
    let base = temp_dir.path();

    fs::create_dir_all(base.join("documents"))?;
    fs::create_dir_all(base.join("images"))?;
    fs::write(base.join("readme.txt"), "Hello")?;
    fs::write(base.join("documents/report.md"), "Report content")?;

    // Create harness
    let mut harness = TuiTestHarness::new(120, 40)?;

    // Spawn scarab
    let mut cmd = CommandBuilder::new("scarab");
    cmd.arg(base);
    harness.spawn(cmd)?;

    // Wait for initial load
    harness.wait_for_text("readme.txt")?;
    harness.wait_for_text("documents")?;

    // Navigate to documents folder
    harness.send_key(KeyCode::Down)?;
    harness.send_key(KeyCode::Enter)?;

    // Verify we're in documents
    harness.wait_for_text("report.md")?;

    // Go back to parent
    harness.send_key(KeyCode::Char('h'))?;

    // Verify we're back
    harness.wait_for_text("readme.txt")?;

    // Search for markdown files
    harness.send_key(KeyCode::Char('/'))?;
    harness.type_text("md")?;

    // Should show nothing in current dir
    assert!(!harness.screen_text().contains("readme.txt"));

    // Clear search and navigate to images
    harness.send_key(KeyCode::Esc)?;
    harness.send_key(KeyCode::Down)?;
    harness.send_key(KeyCode::Down)?;

    // Create snapshot of final state
    harness.assert_snapshot("file_manager_final_state")?;

    Ok(())
}
```

---

## Troubleshooting

### Issue: File list not appearing

**Symptoms**: Test times out waiting for file names

**Solutions**:
- Ensure directory path is correct
- Check that test directory is created before spawning
- Verify file manager has permissions to read directory
- Increase wait timeout for large directories

```rust
// Before
harness.wait_for_text("file.txt")?;

// After - with longer timeout
harness.wait_for_text_with_timeout("file.txt", Duration::from_secs(5))?;
```

---

### Issue: Selection indicator not detected

**Symptoms**: Cannot find selection arrow or highlight

**Solutions**:
- Different file managers use different indicators (`►`, `>`, `*`, etc.)
- Use regex patterns for flexible matching
- Check for highlight attributes instead of symbols

```rust
// Flexible selection detection
harness.wait_until(|| {
    let text = harness.screen_text();
    text.contains("►") || text.contains(">") || text.contains("*")
}, Duration::from_secs(1))?;
```

---

### Issue: Directory navigation timing issues

**Symptoms**: Test fails because directory change hasn't completed

**Solutions**:
- Wait for breadcrumb/path display to update
- Wait for expected file to appear
- Use `wait_for_stable()` after navigation

```rust
// Robust directory navigation
harness.send_key(KeyCode::Enter)?;
harness.wait_for_stable()?; // Wait for UI to stabilize
harness.wait_for_text("expected_file.txt")?; // Verify we're in right dir
```

---

### Issue: Filesystem operations not reflected

**Symptoms**: File deleted in UI but still exists on disk

**Solutions**:
- Add delay for filesystem sync
- Poll for file existence
- Verify operation completion message

```rust
// Wait for filesystem operation
harness.send_key(KeyCode::Char('d'))?;
harness.send_key(KeyCode::Char('y'))?;

// Wait for success message
harness.wait_for_text("Deleted successfully")?;

// Poll for file deletion
std::thread::sleep(Duration::from_millis(100));
assert!(!path.exists());
```

---

## Related Resources

- [Core API Documentation](https://docs.rs/terminal-testlib)
- [Navigation Demo Example](../../../../examples/navigation_demo.rs)
- [Scarab-Nav Cookbook](scarab-nav.md) - Navigation patterns
- [Snapshot Testing Guide](../../../ARCHITECTURE.md#snapshot-testing)

## Contributing

Have a file manager testing pattern to share? Please open a PR with:
- Pattern description
- Working code example
- Expected behavior
- Common pitfalls

See [CONTRIBUTING.md](../../../../CONTRIBUTING.md) for guidelines.

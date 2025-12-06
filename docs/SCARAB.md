# Scarab Testing Guide

> Testing guide for Scarab terminal emulator integration

## Overview

The `scarab` module provides Scarab-specific wrappers around the generic [`ipc`](./IPC.md) module, preconfigured for Scarab's default paths and protocol.

Scarab uses a split architecture:
- **scarab-daemon**: Manages the PTY, parses terminal state, exposes it via shared memory
- **scarab** (GPU client): Renders the UI by reading from shared memory

This module provides the glue to test Scarab without reimplementing IPC/shared-memory plumbing.

## Quick Start

### 1. Enable the Feature

```toml
[dependencies]
ratatui-testlib = { version = "0.3", features = ["scarab"] }
```

### 2. Set Environment Variable

Enable Scarab testing mode:

```bash
export SCARAB_TEST_RTL=1
```

### 3. Basic Test

```rust
use std::time::Duration;
use ratatui_testlib::scarab::ScarabTestHarness;

#[test]
fn test_echo_command() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to running scarab-daemon using default paths
    let mut harness = ScarabTestHarness::connect()?;

    // Send input via IPC
    harness.send_input("echo hello\n")?;

    // Wait for output in shared memory grid
    harness.wait_for_text("hello", Duration::from_secs(5))?;

    // Assert grid contents
    let grid = harness.grid_contents()?;
    assert!(grid.contains("hello"));

    Ok(())
}
```

## Default Configuration

Scarab uses these default paths:

| Setting | Default Value |
|---------|---------------|
| Socket | `/tmp/scarab-daemon.sock` |
| Shared memory | `/scarab_shm_v1` |
| Image buffer | `/scarab_img_v1` |
| Magic number | `0x5343_5241` ("SCRA") |
| Terminal size | 80x24 |

## Custom Configuration

```rust
use std::time::Duration;
use ratatui_testlib::scarab::{ScarabTestHarness, ScarabConfig};

let config = ScarabConfig::builder()
    .socket_path("/tmp/test-scarab.sock")
    .shm_path("/test_scarab_shm")
    .dimensions(120, 40)
    .prompt_patterns(vec!["$ ".to_string(), "# ".to_string()])
    .build();

let mut harness = ScarabTestHarness::with_config(config)?;
```

## Testing Patterns

### Pattern 1: Wait for Shell Prompt

```rust
use std::time::Duration;
use ratatui_testlib::scarab::ScarabTestHarness;

#[test]
fn test_with_prompt() -> Result<(), Box<dyn std::error::Error>> {
    let mut harness = ScarabTestHarness::connect()?;

    // Wait for shell prompt
    harness.wait_for_prompt(Duration::from_secs(5))?;

    // Send command
    harness.send_input("ls -la\n")?;

    // Wait for output and next prompt
    harness.wait_for_text("total", Duration::from_secs(2))?;

    Ok(())
}
```

### Pattern 2: Test Escape Sequences

```rust
use std::time::Duration;
use ratatui_testlib::scarab::ScarabTestHarness;

#[test]
fn test_cursor_movement() -> Result<(), Box<dyn std::error::Error>> {
    let mut harness = ScarabTestHarness::connect()?;

    // Send cursor movement
    harness.send_input("\x1b[A")?; // Up arrow

    // Verify cursor position changed
    let (row, col) = harness.cursor_position()?;
    println!("Cursor at row: {}, col: {}", row, col);

    Ok(())
}
```

### Pattern 3: Test Sequence of Commands

```rust
use std::time::Duration;
use ratatui_testlib::scarab::ScarabTestHarness;

#[test]
fn test_command_sequence() -> Result<(), Box<dyn std::error::Error>> {
    let mut harness = ScarabTestHarness::connect()?;

    harness.send_input("cd /tmp && ls\n")?;

    // Wait for multiple texts in order
    harness.wait_for_sequence(&["cd /tmp", "ls"], Duration::from_secs(5))?;

    Ok(())
}
```

### Pattern 4: Test Text Disappearance

```rust
use std::time::Duration;
use ratatui_testlib::scarab::ScarabTestHarness;

#[test]
fn test_clear_screen() -> Result<(), Box<dyn std::error::Error>> {
    let mut harness = ScarabTestHarness::connect()?;

    // Write something
    harness.send_input("echo 'temporary text'\n")?;
    harness.wait_for_text("temporary text", Duration::from_secs(2))?;

    // Clear screen
    harness.send_input("clear\n")?;

    // Wait for text to disappear
    harness.wait_for_text_absent("temporary text", Duration::from_secs(2))?;

    Ok(())
}
```

## API Reference

### ScarabTestHarness

The main test harness for Scarab testing.

```rust
impl ScarabTestHarness {
    // Connection
    fn is_enabled() -> bool;
    fn connect() -> IpcResult<Self>;
    fn with_config(config: ScarabConfig) -> IpcResult<Self>;

    // Input
    fn send_input(&mut self, text: &str) -> IpcResult<()>;
    fn send_bytes(&mut self, bytes: &[u8]) -> IpcResult<()>;
    fn resize(&mut self, cols: u16, rows: u16) -> IpcResult<()>;

    // State
    fn refresh(&mut self) -> IpcResult<()>;
    fn grid_contents(&self) -> IpcResult<String>;
    fn cursor_position(&self) -> IpcResult<(u16, u16)>;
    fn dimensions(&self) -> (u16, u16);
    fn contains(&self, text: &str) -> IpcResult<bool>;

    // Wait helpers
    fn wait_for_text(&mut self, text: &str, timeout: Duration) -> IpcResult<()>;
    fn wait_for_text_absent(&mut self, text: &str, timeout: Duration) -> IpcResult<()>;
    fn wait_for_prompt(&mut self, timeout: Duration) -> IpcResult<()>;
    fn wait_for_sequence(&mut self, texts: &[&str], timeout: Duration) -> IpcResult<()>;
    fn wait_for_update(&mut self, timeout: Duration) -> IpcResult<()>;

    // Assertions
    fn assert_contains(&self, text: &str) -> IpcResult<()>;
}
```

### ScarabConfig

Configuration options for the test harness.

```rust
impl ScarabConfig {
    fn builder() -> ScarabConfigBuilder;
}

impl ScarabConfigBuilder {
    fn socket_path(self, path: impl Into<PathBuf>) -> Self;
    fn shm_path(self, path: impl Into<String>) -> Self;
    fn image_shm_path(self, path: impl Into<String>) -> Self;
    fn dimensions(self, cols: u16, rows: u16) -> Self;
    fn connect_timeout(self, timeout: Duration) -> Self;
    fn default_timeout(self, timeout: Duration) -> Self;
    fn prompt_patterns(self, patterns: Vec<String>) -> Self;
    fn add_prompt_pattern(self, pattern: impl Into<String>) -> Self;
    fn build(self) -> ScarabConfig;
}
```

## CI/CD Integration

### GitHub Actions Example

```yaml
jobs:
  test-scarab:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Build scarab-daemon
        run: cargo build --release -p scarab-daemon

      - name: Start daemon
        run: |
          ./target/release/scarab-daemon &
          sleep 2

      - name: Run Scarab tests
        env:
          SCARAB_TEST_RTL: "1"
        run: cargo test --features scarab
```

## Troubleshooting

### Issue: "SCARAB_TEST_RTL environment variable not set"

**Solution**: Set the environment variable:
```bash
export SCARAB_TEST_RTL=1
```

### Issue: Socket Not Found

**Symptoms**: `IpcError::SocketNotFound`

**Solutions**:
- Verify scarab-daemon is running
- Check socket path matches configuration:
  ```bash
  ls -la /tmp/scarab-daemon.sock
  ```

### Issue: Invalid Magic Number

**Symptoms**: `IpcError::InvalidData` with magic mismatch

**Solutions**:
- Ensure scarab-daemon version matches
- Scarab uses magic `0x5343_5241` ("SCRA")
- Check for protocol version mismatches

### Issue: Timeout Waiting for Text

**Symptoms**: `IpcError::Timeout`

**Solutions**:
- Increase timeout duration
- Verify daemon is processing input
- Check shared memory is updating:
  ```rust
  // Check sequence number changes
  let seq1 = harness.shared_memory().sequence_number();
  std::thread::sleep(Duration::from_millis(100));
  harness.refresh()?;
  let seq2 = harness.shared_memory().sequence_number();
  assert_ne!(seq1, seq2, "Shared memory not updating");
  ```

## Related Resources

- [IPC Module Documentation](./IPC.md) - Generic IPC helpers
- [Scarab GitHub Repository](https://github.com/raibid-labs/scarab) - Scarab terminal emulator
- [Examples](../examples/scarab_test.rs) - Working example code

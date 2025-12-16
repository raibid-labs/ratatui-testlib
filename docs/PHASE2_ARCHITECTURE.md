# Phase 2 Architecture: Event Simulation & Async Support

**Status**: Final Design
**Date**: 2025-11-20
**Phase**: 2 of 6 (MVP)

---

## Executive Summary

Phase 2 transforms terminal-testlib from a passive observation tool to an active interaction framework. This document outlines the architectural decisions for event simulation and async runtime integration.

### Key Decisions

1. **Event Model**: VT100 escape sequence generation from high-level key events
2. **Async Strategy**: Implement native `AsyncTuiTestHarness` (Option A)
3. **Wait Mechanism**: Polling with configurable intervals and timeouts
4. **API Design**: Ergonomic, intuitive methods mirroring common testing patterns

---

## 1. Event Simulation Architecture

### 1.1 Design Goals

- **Ergonomic API**: Simple, intuitive methods for common operations
- **Complete Coverage**: Support all keyboard events needed for TUI testing
- **Terminal Agnostic**: Generate correct escape sequences for standard terminals
- **Type Safe**: Prevent invalid key combinations at compile time
- **Extensible**: Easy to add new keys or custom escape sequences

### 1.2 Event Type Hierarchy

```
KeyEvent
├── code: KeyCode
└── modifiers: Modifiers

KeyCode (enum)
├── Char(char)          - Alphanumeric and symbols
├── Enter               - Line feed
├── Esc                 - Escape
├── Tab                 - Horizontal tab
├── Backspace           - Delete previous character
├── Delete              - Delete current character
├── Up/Down/Left/Right  - Navigation arrows
├── Home/End            - Line navigation
├── PageUp/PageDown     - Page navigation
├── Insert              - Insert mode toggle
└── F(u8)               - Function keys F1-F12

Modifiers (bitflags)
├── SHIFT               - Shift key
├── CTRL                - Control key
├── ALT                 - Alt/Option key
└── META                - Meta/Cmd/Windows key
```

### 1.3 Escape Sequence Mapping

**Design Decision**: Direct mapping to VT100/ANSI escape sequences

**Rationale**:
- Standard VT100 sequences work across all modern terminals
- Direct control over exact bytes sent to PTY
- No dependency on terminal detection or capabilities
- Matches behavior of real terminal input

**Implementation Strategy**:

```rust
impl KeyCode {
    pub fn to_escape_sequence(&self) -> Vec<u8> {
        match self {
            // Printable characters
            KeyCode::Char(c) => c.to_string().into_bytes(),

            // Control characters
            KeyCode::Enter => b"\n".to_vec(),
            KeyCode::Tab => b"\t".to_vec(),
            KeyCode::Esc => b"\x1b".to_vec(),
            KeyCode::Backspace => b"\x7f".to_vec(),

            // CSI sequences (ESC [)
            KeyCode::Up => b"\x1b[A".to_vec(),
            KeyCode::Down => b"\x1b[B".to_vec(),
            KeyCode::Right => b"\x1b[C".to_vec(),
            KeyCode::Left => b"\x1b[D".to_vec(),
            KeyCode::Home => b"\x1b[H".to_vec(),
            KeyCode::End => b"\x1b[F".to_vec(),

            // SS3 sequences (ESC O)
            KeyCode::F(1) => b"\x1bOP".to_vec(),
            KeyCode::F(2) => b"\x1bOQ".to_vec(),
            // ... etc
        }
    }
}
```

**Modifier Handling**:

```rust
impl KeyEvent {
    pub fn to_escape_sequence(&self) -> Vec<u8> {
        match (self.code, self.modifiers) {
            // Ctrl combinations
            (KeyCode::Char(c), m) if m.contains(Modifiers::CTRL) => {
                // Ctrl+A = 0x01, Ctrl+B = 0x02, etc.
                vec![ctrl_char(c)]
            }

            // Alt combinations
            (KeyCode::Char(c), m) if m.contains(Modifiers::ALT) => {
                // Alt+key = ESC + key
                let mut seq = b"\x1b".to_vec();
                seq.push(c as u8);
                seq
            }

            // No modifiers or unsupported combination
            _ => self.code.to_escape_sequence()
        }
    }
}
```

### 1.4 API Design

**Core Methods**:

```rust
impl TuiTestHarness {
    /// Send a single key event (no modifiers)
    pub fn send_key(&mut self, key: KeyCode) -> Result<()>;

    /// Send a key with modifiers (Ctrl, Alt, etc.)
    pub fn send_key_with_modifiers(
        &mut self,
        key: KeyCode,
        modifiers: Modifiers,
    ) -> Result<()>;

    /// Type a text string (convenience for multiple Char events)
    pub fn send_keys(&mut self, text: &str) -> Result<()>;
}
```

**Design Rationale**:
- `send_key()`: Simple, common case (single key press)
- `send_key_with_modifiers()`: Explicit about modifiers, type-safe
- `send_keys()`: Convenience for text input, mirrors common test patterns

**Alternative Considered**: Single `send_key(KeyEvent)` method
- **Rejected**: Less ergonomic for common case, requires constructing KeyEvent

---

## 2. Wait Conditions Architecture

### 2.1 Design Goals

- **Reliable**: Must handle timing variations in TUI apps
- **Flexible**: Support arbitrary conditions
- **Ergonomic**: Common patterns should be easy
- **Debuggable**: Failed waits should provide actionable information
- **Configurable**: Timeouts and polling intervals tunable per-test

### 2.2 Polling Strategy

**Design Decision**: Active polling with configurable intervals

**Rationale**:
- Simple, predictable behavior
- Works with any PTY-based application
- No dependency on signal/event mechanisms
- Easy to tune for performance vs accuracy

**Architecture**:

```
┌─────────────────────────────────────────────────┐
│ wait_for(condition)                              │
│                                                  │
│  ┌──────────────────────────────────────────┐   │
│  │ Loop until condition true or timeout     │   │
│  │                                          │   │
│  │  1. update_state()                       │   │
│  │     └─ Read PTY output                   │   │
│  │     └─ Feed to vtparse                   │   │
│  │     └─ Update ScreenState                │   │
│  │                                          │   │
│  │  2. Check condition(state)               │   │
│  │     └─ Return Ok(()) if true             │   │
│  │                                          │   │
│  │  3. Check elapsed time                   │   │
│  │     └─ Return Err if timeout             │   │
│  │                                          │   │
│  │  4. Sleep(poll_interval)                 │   │
│  │                                          │   │
│  └──────────────────────────────────────────┘   │
└─────────────────────────────────────────────────┘
```

**Configuration**:

```rust
pub struct TuiTestHarness {
    timeout: Duration,        // Default: 5 seconds
    poll_interval: Duration,  // Default: 100ms
    // ...
}
```

**Tuning Guidelines**:
- **Fast tests**: 50ms poll interval, 2s timeout
- **Standard tests**: 100ms poll interval, 5s timeout
- **Slow apps**: 200ms poll interval, 10s timeout

### 2.3 Condition API

**Design Decision**: Closure-based conditions for maximum flexibility

```rust
pub fn wait_for<F>(&mut self, condition: F) -> Result<()>
where
    F: Fn(&ScreenState) -> bool
```

**Convenience Methods**:

```rust
impl TuiTestHarness {
    /// Wait for specific text anywhere on screen
    pub fn wait_for_text(&mut self, text: &str) -> Result<()> {
        let text = text.to_string();
        self.wait_for(|state| state.contains(&text))
    }

    /// Wait for cursor to reach position
    pub fn wait_for_cursor(&mut self, row: u16, col: u16) -> Result<()> {
        self.wait_for(|state| {
            state.cursor_position() == (row, col)
        })
    }
}
```

**Common Patterns Module** (optional):

```rust
pub mod wait {
    /// Wait for text to appear at specific position
    pub fn text_at(row: u16, col: u16, text: &str)
        -> impl Fn(&ScreenState) -> bool;

    /// Wait for text to disappear
    pub fn text_disappears(text: &str)
        -> impl Fn(&ScreenState) -> bool;

    /// Wait for screen to change from previous state
    pub fn screen_changed(prev: &str)
        -> impl Fn(&ScreenState) -> bool;
}
```

### 2.4 Error Handling

**Design Decision**: Rich error context on timeout

```rust
pub enum TermTestError {
    Timeout {
        timeout_ms: u64,
        // Future: Add context about what was waited for
    },
    // ... other errors
}
```

**Debug Output on Timeout**:

```rust
eprintln!("\n=== Timeout waiting for: {} ===", description);
eprintln!("Waited: {:?} ({} iterations)", elapsed, iterations);
eprintln!("Cursor position: row={}, col={}", cursor.0, cursor.1);
eprintln!("Current screen state:\n{}", current_state);
eprintln!("==========================================\n");
```

**Rationale**:
- Helps developers debug why wait failed
- Shows actual screen state vs expected
- Provides timing information for tuning

---

## 3. Async Architecture

### 3.1 Strategy Selection

**Decision**: Implement native AsyncTuiTestHarness (Option A)

**Options Considered**:

| Aspect | Option A: Native Async | Option B: Sync in Async |
|--------|------------------------|------------------------|
| **Ergonomics** | Excellent (async/await) | Good (manual wrapping) |
| **Performance** | Good (true async I/O) | Fair (blocking calls) |
| **Complexity** | Higher (new harness) | Lower (reuse existing) |
| **Maintenance** | Medium (two harnesses) | Low (one harness) |
| **Future-proof** | Yes (async-first) | Limited (retrofit) |

**Decision Rationale**:
- Async is first-class requirement for dgx-pixels (Tokio + Bevy)
- Native async provides better ergonomics for async tests
- One-time complexity investment for long-term benefits
- Can share implementation with sync harness (common core)

### 3.2 AsyncTuiTestHarness Design

**Architecture**:

```rust
#[cfg(feature = "async-tokio")]
pub struct AsyncTuiTestHarness {
    // Option 1: Wrap sync harness
    inner: TuiTestHarness,

    // Option 2: Own PTY layer (future optimization)
    // pty: AsyncTestTerminal,
    // state: ScreenState,
}
```

**Implementation Strategy**: Start with Option 1 (wrapping), optimize to Option 2 if needed

**Async Methods**:

```rust
impl AsyncTuiTestHarness {
    pub async fn new(width: u16, height: u16) -> Result<Self>;

    pub async fn spawn(&mut self, cmd: CommandBuilder) -> Result<()>;

    pub async fn send_key(&mut self, key: KeyCode) -> Result<()>;

    pub async fn send_keys(&mut self, text: &str) -> Result<()>;

    pub async fn wait_for<F>(&mut self, condition: F) -> Result<()>
    where
        F: Fn(&ScreenState) -> bool;

    pub async fn update_state(&mut self) -> Result<()>;
}
```

### 3.3 Async Wait Implementation

**Design Decision**: Use tokio::time for async sleeps and timeouts

```rust
pub async fn wait_for<F>(&mut self, condition: F) -> Result<()>
where
    F: Fn(&ScreenState) -> bool,
{
    let timeout = self.inner.timeout;
    let poll_interval = self.inner.poll_interval;

    tokio::time::timeout(timeout, async {
        loop {
            // Update state (may block, wrap if needed)
            self.update_state().await?;

            if condition(&self.inner.state) {
                return Ok(());
            }

            // Async sleep (yields to runtime)
            tokio::time::sleep(poll_interval).await;
        }
    })
    .await
    .map_err(|_| TermTestError::Timeout {
        timeout_ms: timeout.as_millis() as u64,
    })?
}
```

**Key Points**:
- `tokio::time::sleep`: Cooperative multitasking
- `tokio::time::timeout`: First-class timeout support
- Minimal changes to core logic

### 3.4 Blocking I/O Handling

**Challenge**: PTY I/O operations are blocking

**Solution**: Wrap blocking calls in `spawn_blocking`

```rust
pub async fn spawn(&mut self, cmd: CommandBuilder) -> Result<()> {
    let terminal = &mut self.inner.terminal;

    // Move blocking work to thread pool
    let result = tokio::task::spawn_blocking(move || {
        terminal.spawn(cmd)
    }).await;

    result.map_err(|e| TermTestError::from(e))?
}
```

**Alternative Considered**: Async PTY library
- **Rejected**: No mature async PTY library, would require custom implementation
- **Future**: Could implement AsyncTestTerminal using tokio::io if needed

---

## 4. Module Structure

### 4.1 New Modules

```
src/
├── lib.rs              (update exports)
├── events.rs           (NEW - KeyCode, Modifiers, KeyEvent)
├── harness.rs          (update - add event methods)
├── async_harness.rs    (NEW - AsyncTuiTestHarness)
├── wait.rs             (NEW - wait condition helpers)
├── pty.rs              (existing)
├── screen.rs           (existing)
└── error.rs            (existing)
```

### 4.2 Public API Surface

**Exports from lib.rs**:

```rust
// Event types
pub use events::{KeyCode, Modifiers, KeyEvent};

// Sync harness
pub use harness::TuiTestHarness;

// Async harness (feature-gated)
#[cfg(feature = "async-tokio")]
pub use async_harness::AsyncTuiTestHarness;

// Wait helpers (optional, may be methods only)
pub mod wait;
```

### 4.3 Feature Flags

**No changes needed** - async-tokio feature already exists:

```toml
[features]
async-tokio = ["tokio"]
```

**Additional dependency needed**:

```toml
[dependencies]
bitflags = "2.4"
```

---

## 5. Testing Strategy

### 5.1 Test Coverage Plan

| Component | Test Type | Coverage Target |
|-----------|-----------|-----------------|
| KeyCode | Unit | 100% (enum variants) |
| Escape sequences | Unit | 100% (all mappings) |
| Event sending | Integration | 90% (common keys) |
| Wait conditions | Integration | 90% (success/timeout) |
| Async harness | Integration | 80% (async scenarios) |

### 5.2 Test Organization

```
tests/
├── integration/
│   ├── events.rs          (event simulation tests)
│   ├── wait.rs            (wait condition tests)
│   └── async.rs           (async harness tests)
└── unit/
    └── escape_sequences.rs (escape mapping tests)
```

### 5.3 CI Strategy

**Test Matrix**:

| Configuration | Features | Purpose |
|---------------|----------|---------|
| Minimal | (none) | Core functionality |
| Async | async-tokio | Async support |
| Full | mvp | Complete feature set |

**CI Commands**:

```bash
# Core tests (no features)
cargo test --lib

# Async tests
cargo test --lib --features async-tokio

# Integration tests
cargo test --test '*'

# Examples (smoke tests)
cargo run --example keyboard_events
cargo run --example async_test --features async-tokio
```

---

## 6. Performance Considerations

### 6.1 Event Sending Overhead

**Expected**: < 1ms per event
- Escape sequence generation: < 0.1ms (simple match)
- PTY write: < 0.5ms (syscall overhead)
- Screen state update: < 0.5ms (vtparse overhead)

**Optimization Opportunities**:
- Cache escape sequences for common keys
- Batch multiple events before update
- Use static byte arrays instead of Vec

### 6.2 Wait Condition Polling

**Expected**: 10-20ms per iteration
- update_state(): 5-10ms (PTY read + parse)
- Condition check: < 1ms (screen contents search)
- Sleep overhead: configurable (default 100ms)

**Tuning Strategies**:
- Reduce poll_interval for fast tests (50ms)
- Increase poll_interval to reduce CPU (200ms)
- Consider exponential backoff for long waits

### 6.3 Async Overhead

**Expected**: Minimal (< 5% vs sync)
- spawn_blocking overhead: < 1ms per call
- tokio::time overhead: < 0.1ms per sleep
- Benefit: Concurrent test execution

---

## 7. API Examples

### 7.1 Basic Event Simulation

```rust
#[test]
fn test_simple_input() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;
    harness.spawn(CommandBuilder::new("my-app"))?;

    // Wait for prompt
    harness.wait_for_text("Enter name:")?;

    // Type text
    harness.send_keys("Alice")?;
    harness.send_key(KeyCode::Enter)?;

    // Verify response
    harness.wait_for_text("Hello, Alice!")?;
    Ok(())
}
```

### 7.2 Navigation Keys

```rust
#[test]
fn test_navigation() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;
    harness.spawn(CommandBuilder::new("menu-app"))?;

    // Navigate down
    harness.send_key(KeyCode::Down)?;
    harness.wait_for_text("> Option 2")?;

    // Select with Enter
    harness.send_key(KeyCode::Enter)?;
    harness.wait_for_text("Selected: Option 2")?;

    Ok(())
}
```

### 7.3 Modifiers

```rust
#[test]
fn test_ctrl_keys() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;
    harness.spawn(CommandBuilder::new("cat"))?;

    // Type some text
    harness.send_keys("Hello")?;

    // Send EOF (Ctrl+D)
    harness.send_key_with_modifiers(
        KeyCode::Char('d'),
        Modifiers::CTRL
    )?;

    // cat should exit
    harness.wait_exit()?;
    Ok(())
}
```

### 7.4 Async Testing

```rust
#[tokio::test]
async fn test_async_wait() -> Result<()> {
    let mut harness = AsyncTuiTestHarness::new(80, 24).await?;
    harness.spawn(CommandBuilder::new("my-app")).await?;

    // Async wait
    harness.wait_for(|state| {
        state.contains("Ready")
    }).await?;

    // Async event sending
    harness.send_key(KeyCode::Enter).await?;

    Ok(())
}
```

---

## 8. Migration Path

### 8.1 From Phase 1 to Phase 2

**For Users**:

1. Update imports:
   ```rust
   use term_test::{TuiTestHarness, KeyCode};
   ```

2. Replace `send_text()` with `send_keys()`:
   ```rust
   // Old
   harness.send_text("hello\n")?;

   // New
   harness.send_keys("hello")?;
   harness.send_key(KeyCode::Enter)?;
   ```

3. Use typed keys instead of strings:
   ```rust
   // More explicit and type-safe
   harness.send_key(KeyCode::Tab)?;
   harness.send_key(KeyCode::Char('2'))?;
   ```

**Backwards Compatibility**: Keep `send_text()` as deprecated alias

---

## 9. Future Enhancements

### 9.1 Phase 2 Extensions (Post-MVP)

**Mouse Events**:
```rust
pub enum MouseEvent {
    Click { row: u16, col: u16, button: MouseButton },
    Drag { from: (u16, u16), to: (u16, u16) },
    Scroll { direction: ScrollDirection },
}
```

**Resize Events**:
```rust
pub fn send_resize(&mut self, width: u16, height: u16) -> Result<()>;
```

**Paste Events**:
```rust
pub fn send_paste(&mut self, text: &str) -> Result<()>;
```

### 9.2 Advanced Wait Patterns

**Retry Logic**:
```rust
pub fn wait_with_retry<F>(&mut self, condition: F, retries: u32) -> Result<()>;
```

**Change Detection**:
```rust
pub fn wait_for_change(&mut self) -> Result<()>;
```

**Multiple Conditions**:
```rust
pub fn wait_for_any(&mut self, conditions: Vec<Box<dyn Fn(&ScreenState) -> bool>>) -> Result<()>;
```

---

## 10. Risks and Mitigations

### 10.1 Identified Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Terminal incompatibility** | High | Test on multiple terminals, reference VT100 spec |
| **Timing flakiness** | Medium | Generous timeouts, retry logic, document tuning |
| **Async complexity** | Medium | Start simple (wrap sync), optimize iteratively |
| **Escape sequence bugs** | High | Comprehensive tests, reference implementations |

### 10.2 Validation Strategy

1. **Unit tests**: Verify escape sequence correctness
2. **Integration tests**: Test against real commands (cat, echo, etc.)
3. **Example validation**: Run examples in CI
4. **Manual testing**: Test with actual TUI apps
5. **Reference checks**: Compare with crossterm, termion implementations

---

## Conclusion

Phase 2 architecture provides:

1. **Complete event simulation** via VT100 escape sequences
2. **Flexible wait conditions** with timeout and polling
3. **Native async support** with AsyncTuiTestHarness
4. **Ergonomic API** for common testing patterns
5. **Strong foundation** for Phase 3 (Sixel) and Phase 4 (Bevy)

The design balances simplicity, flexibility, and performance while maintaining backwards compatibility and supporting the dgx-pixels use case.

**Next Steps**: Begin implementation following PHASE2_CHECKLIST.md

---

**Document Status**: Final
**Approved By**: Studio Producer
**Implementation Ready**: Yes

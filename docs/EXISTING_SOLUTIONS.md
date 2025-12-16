# Existing Ratatui Testing Solutions

## Overview

This document catalogs existing testing solutions and approaches for Ratatui TUI applications, analyzing their strengths, limitations, and how `terminal-testlib` fits into the ecosystem.

## Ratatui Built-in Testing: TestBackend

### What It Is

`TestBackend` is a `Backend` implementation included with Ratatui specifically designed for testing. It renders to an in-memory buffer instead of an actual terminal.

### Basic Usage

```rust
use ratatui::{backend::TestBackend, Terminal, Frame};

#[test]
fn test_widget_rendering() {
    let backend = TestBackend::new(80, 30);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        // Your rendering code here
    }).unwrap();

    let backend = terminal.backend();
    // Access the buffer to make assertions
}
```

### Key Features

- **In-memory rendering**: No actual terminal required
- **Buffer access**: Can access the rendered buffer directly
- **Deterministic**: Same code always produces same output
- **Fast**: No I/O overhead
- **Error handling**: Uses `core::convert::Infallible` for error type (recent versions)

### Integration Testing Pattern

The most valuable test pattern is testing against `TestBackend` to assert the content of the output buffer that would have been flushed to the terminal after a draw call. See `widgets_block_renders` in `tests/widgets_block.rs` in the Ratatui repository for examples.

### Limitations

1. **Not user-friendly**: Acknowledged by Ratatui maintainers in Discussion #78
2. **Low-level**: Requires manual buffer inspection
3. **No PTY**: Can't test PTY-specific behaviors
4. **No graphics**: Can't test Sixel or other graphics protocols
5. **Text-only**: Limited to character grid representation

### Best Use Cases

- Unit testing individual widgets
- Testing layout calculations
- Testing rendering logic in isolation
- Fast, deterministic CI tests

### Official Documentation

- [Ratatui Backend Docs](https://docs.rs/ratatui/latest/ratatui/backend/)
- [Ratatui Contributing Guide](https://github.com/ratatui/ratatui/blob/main/CONTRIBUTING.md)

## Snapshot Testing with insta

### Overview

Ratatui officially recommends using the `insta` crate for snapshot testing of TUI applications.

### Official Recipe

Ratatui provides an official recipe at https://ratatui.rs/recipes/testing/snapshots/

### How It Works

1. Render your UI to a `TestBackend`
2. Capture the buffer as a string (each line represents a row)
3. Use `insta::assert_snapshot!()` to compare with saved snapshots
4. Review changes with `cargo-insta` tool

### Example

```rust
use ratatui::{backend::TestBackend, Terminal};
use insta::assert_snapshot;

#[test]
fn test_menu_layout() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render_menu(f);
    }).unwrap();

    let buffer = terminal.backend().buffer().clone();
    assert_snapshot!(buffer);
}
```

### Features

- **Batteries included**: Multiple snapshot formats, review tools
- **Auto-update**: `cargo insta review` to accept changes
- **IDE integration**: Various editor plugins available
- **Filtering**: Can ignore specific parts of output
- **Inline snapshots**: Can store snapshots in source code

### Workflow

1. Write test with `assert_snapshot!()`
2. Run test (fails initially)
3. Run `cargo insta review`
4. Accept or reject snapshot
5. Snapshot saved, future tests compare against it

### Limitations

- Still uses `TestBackend` under the hood (inherits its limitations)
- No PTY or graphics testing
- Snapshot files can become large for complex UIs

### Alternative: expect-test

The `expect-test` crate is a minimalist alternative:

```rust
use expect_test::expect;

#[test]
fn test_output() {
    let output = render_to_string();
    expect![[r#"
        Expected output
        goes here
    "#]].assert_eq(&output);
}
```

**Key Difference**: Snapshots stored in source code, auto-updated with `UPDATE_EXPECT=1` env var.

**Trade-offs**:
- `expect-test`: Lighter weight, snapshots in code
- `insta`: More features, separate snapshot files

## term-transcript: CLI/REPL Snapshot Testing

### Repository

https://slowli.github.io/term-transcript/term_transcript/

### What It Is

A crate for snapshot testing CLI and REPL applications with support for:
- Capturing terminal interactions with ANSI color information
- SVG rendering for documentation
- Validation testing of terminal output

### Key Features

1. **Transcript Creation**: Captures user-terminal interaction sessions
2. **SVG Output**: Renders transcripts as static SVG images
3. **SVG Parsing**: Can read transcripts back from SVG files
4. **Template-based**: Uses Handlebars for customizable SVG output

### PTY Support

The `portable-pty` feature (optional) enables pseudo-terminal capture:

```rust
use term_transcript::{Transcript, ShellOptions};

let transcript = Transcript::from_inputs(
    &mut ShellOptions::default(),
    vec!["echo 'Hello'", "ls -la"]
)?;
```

### Significant Limitation

**Quote from docs**: "Since most escape sequences are dropped, complex outputs involving cursor movement aren't suitable for capture."

This means **term-transcript cannot test Sixel or complex TUI applications** - it's designed for simpler CLI tools.

### Use Cases

- Testing CLI applications with simple output
- Generating documentation with SVG terminal screenshots
- REPL testing (interactive prompts)
- Simple command-line tools

### Not Suitable For

- Complex TUI applications (cursor movement)
- Ratatui applications (full-screen)
- Sixel or graphics rendering
- Mouse events
- Advanced escape sequences

### Key Types

```rust
struct Transcript {
    // User-terminal interaction
}

struct UserInput {
    // Individual commands
}

struct ShellOptions {
    // Configuration
}

struct Captured {
    // Terminal output with colors
}
```

## tui-term: Pseudoterminal Widget for Ratatui

### Repository

https://github.com/a-kenji/tui-term
https://crates.io/crates/tui-term

### What It Is

A **pseudoterminal widget library** for Ratatui - allows embedding a terminal emulator **inside** a Ratatui application.

### Purpose

This is NOT a testing library. It's for building applications that need to display terminal output as a widget, like:
- Terminal multiplexers (tmux-like)
- IDE integrated terminals
- SSH/remote terminal viewers

### Architecture

- Uses `vt100` crate for parsing terminal control sequences
- Integrates with Ratatui's widget system
- Supports Ratatui theming, blocks, borders

### Example Use Case

```rust
use tui_term::widget::PseudoTerminal;
use vt100::Parser;

let screen = Parser::new(rows, cols);
let widget = PseudoTerminal::new(&screen)
    .block(Block::default().borders(Borders::ALL));

f.render_widget(widget, area);
```

### Relevance to Testing

**Limited**: While it demonstrates how to integrate `vt100` with Ratatui, it's designed for embedding terminals in TUI apps, not for testing TUI apps themselves.

**Could be useful**: The widget implementation shows patterns for working with vt100 screen state, which could inform test harness design.

### Current Limitations

- Only supports `vt100` backend (per docs)
- Development dependencies include `insta` for snapshot testing of the widget itself
- Version 0.2.0, documentation coverage 60.98%

## Ratatui.cs: .NET Bindings with Headless Testing

### Repository

https://github.com/holo-q/Ratatui.cs

### What It Is

.NET bindings for Ratatui, **notable for its headless testing approach**.

### Key Testing Features

1. **Headless snapshot rendering**: No terminal required
2. **CI/smoke tests**: Production-ready for automated testing
3. **Deterministic results**: No flakiness
4. **Retained-mode widget model**: Different from native Ratatui

### Relevance to Rust

While this is a .NET library, it demonstrates the **concept** of headless Ratatui testing. Key takeaway: It's possible to create a test backend that doesn't require a PTY or actual terminal.

### Lessons for terminal-testlib

- Headless testing is valuable for CI/CD
- Could complement PTY-based testing
- Suggests Ratatui could support alternative backends more easily

## Discussion #78: Test Driven Development Support

### Source

https://github.com/ratatui-org/ratatui/discussions/78

### Key Points

- **Acknowledged gap**: "Currently [testing] is hard"
- **TestBackend exists**: But "it's not really user friendly"
- **Community desire**: Better testing framework for Ratatui
- **Open question**: What would an ideal testing framework look like?

### Implications for terminal-testlib

This discussion confirms:
1. There's a real need for better Ratatui testing solutions
2. TestBackend alone is insufficient
3. Community would welcome a comprehensive testing library

## CLI Testing Tools (General Rust)

### assert_cmd

**Purpose**: Testing CLI applications
**Crates.io**: https://crates.io/crates/assert_cmd

```rust
use assert_cmd::Command;

#[test]
fn test_cli() {
    Command::cargo_bin("my-app")
        .unwrap()
        .assert()
        .success();
}
```

**Use case**: Testing that a binary runs, checking exit codes, stdout/stderr

**Limitation**: Not suitable for interactive TUI applications

### assert_fs

**Purpose**: Creating temporary files/directories for tests
**Often used with**: assert_cmd

```rust
use assert_fs::prelude::*;

let temp = assert_fs::TempDir::new()?;
let input_file = temp.child("input.txt");
input_file.write_str("test data")?;
```

**Relevance**: Useful for creating test fixtures (e.g., Sixel image files)

## Comparison Matrix

| Solution | Type | PTY Support | Graphics | Snapshot | Integration Testing | Widget Testing |
|----------|------|-------------|----------|----------|---------------------|----------------|
| **TestBackend** | Built-in | ❌ | ❌ | Via insta | ❌ | ✅ |
| **insta** | Snapshot | Depends on backend | Depends on backend | ✅ | Partial | ✅ |
| **expect-test** | Snapshot | Depends on backend | Depends on backend | ✅ | Partial | ✅ |
| **term-transcript** | CLI testing | ⚠️ Limited | ❌ | ✅ | ❌ (CLI only) | ❌ |
| **tui-term** | Widget | N/A (is a widget) | Depends on vt100 | N/A | ❌ | ❌ |
| **assert_cmd** | CLI testing | ❌ | ❌ | ❌ | ❌ | ❌ |
| **terminal-testlib** (proposed) | Integration | ✅ | ✅ Sixel | ✅ | ✅ | ✅ |

## Gap Analysis: What's Missing?

### 1. PTY-Based Integration Testing

**Gap**: No existing solution tests Ratatui apps in a real PTY environment.

**Why it matters**:
- PTY behavior differs from pipes
- Terminal size negotiation
- Signal handling (SIGWINCH, etc.)
- TTY detection logic

### 2. Graphics Protocol Testing

**Gap**: None of the existing solutions test Sixel or other graphics protocols.

**Why it matters**:
- Sixel is increasingly supported
- Graphics-heavy TUIs need testing
- Image rendering correctness

### 3. User-Friendly Integration Testing

**Gap**: TestBackend requires manual buffer inspection, not ergonomic.

**Why it matters**:
- Developer experience
- Test maintainability
- Adoption barriers

### 4. End-to-End Testing

**Gap**: No solution tests the complete user experience (input → rendering → output).

**Why it matters**:
- Integration bugs
- Event handling
- State management across interactions

### 5. Async TUI Testing

**Gap**: No async-aware testing harness.

**Why it matters**:
- Modern Ratatui apps use Tokio/async-std
- Need to test async event loops
- Race conditions and timing issues

## How terminal-testlib Fits In

### Complementary, Not Competitive

`terminal-testlib` **complements** existing solutions:

| Testing Level | Use This | For What |
|---------------|----------|----------|
| **Unit Tests** | TestBackend + insta | Individual widgets, layout calculations |
| **Integration Tests** | **terminal-testlib** | Full app behavior, PTY interaction, graphics |
| **CLI Tests** | assert_cmd | Binary execution, exit codes |
| **Snapshot Tests** | insta or expect-test | Both unit and integration level |

### Unique Value Proposition

What `terminal-testlib` provides that nothing else does:

1. **PTY-based testing**: Real terminal environment
2. **Sixel support**: Graphics protocol testing
3. **User-friendly API**: Ergonomic test harness
4. **Event simulation**: Keyboard, mouse, resize events
5. **Async support**: Tokio/async-std integration
6. **Full integration**: End-to-end user experience testing

### Migration Path

Existing projects can adopt incrementally:

```rust
// Keep existing unit tests
#[test]
fn test_widget() {
    let backend = TestBackend::new(80, 24);
    // ... existing test ...
}

// Add integration tests
#[test]
fn test_app_integration() {
    let mut harness = TuiTestHarness::new(80, 24).unwrap();
    harness.spawn(Command::new("./my-app")).unwrap();
    // ... new integration test ...
}
```

## Best Practices: When to Use What

### Use TestBackend + insta When:

- Testing widget rendering logic
- Testing layout calculations
- Fast, deterministic CI tests needed
- No PTY or graphics required
- Unit testing individual components

### Use terminal-testlib When:

- Testing full application behavior
- Testing PTY-specific features
- Testing Sixel or graphics rendering
- Simulating user interactions
- Testing async event loops
- Integration testing complete flows

### Use term-transcript When:

- Testing simple CLI tools (not full TUIs)
- Generating documentation with SVG
- Testing REPL interactions
- Simple command-line output verification

### Use assert_cmd When:

- Testing binary execution
- Checking exit codes
- Testing non-interactive CLI tools
- Shell script-like testing

## Code Reuse Opportunities

Components from existing solutions that terminal-testlib can leverage:

1. **vt100 crate**: Already proven in tui-term, good for screen parsing
2. **portable-pty**: From WezTerm, cross-platform, well-maintained
3. **insta**: For snapshot assertions in integration tests
4. **expect-test**: Alternative lightweight snapshot testing

## Community Feedback and Needs

From Ratatui community discussions:

1. **Testing is currently hard** ✅ terminal-testlib addresses this
2. **TestBackend not user-friendly** ✅ terminal-testlib provides better API
3. **Need for integration testing** ✅ terminal-testlib's core purpose
4. **Graphics testing missing** ✅ terminal-testlib includes Sixel support

## References

- [Ratatui Testing Recipes](https://ratatui.rs/recipes/testing/snapshots/)
- [Ratatui TDD Discussion #78](https://github.com/ratatui-org/ratatui/discussions/78)
- [term-transcript Documentation](https://slowli.github.io/term-transcript/term_transcript/)
- [tui-term Repository](https://github.com/a-kenji/tui-term)
- [Ratatui.cs GitHub](https://github.com/holo-q/Ratatui.cs)
- [Testing TUI Apps Blog](https://blog.waleedkhan.name/testing-tui-apps/)
- [Integration Testing TUI in Rust](https://quantonganh.com/2024/01/21/integration-testing-tui-app-in-rust.md)

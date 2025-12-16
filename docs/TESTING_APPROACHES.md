# Testing Approaches for Terminal User Interfaces

## Overview

This document outlines different testing methodologies for terminal user interface (TUI) applications, their trade-offs, and when to apply each approach.

## The Testing Pyramid for TUI Applications

```
           ╱╲
          ╱  ╲      E2E Tests (few)
         ╱────╲     - Full PTY integration
        ╱      ╲    - Real user scenarios
       ╱────────╲   - Graphics/Sixel
      ╱          ╲  Integration Tests (some)
     ╱────────────╲ - Component interaction
    ╱              ╲- Event handling
   ╱────────────────╲ Unit Tests (many)
  ╱                  ╲ - Widget logic
 ╱____________________╲- Layout calculations
                         - Pure functions
```

## 1. Unit Testing

### What to Test

- Individual widget rendering
- Layout calculations
- State management logic
- Pure functions (formatting, validation, etc.)
- Business logic separate from UI

### Tools

- Ratatui's `TestBackend`
- Standard Rust testing (`#[test]`)
- Assertion libraries

### Example

```rust
use ratatui::{backend::TestBackend, Terminal, widgets::Paragraph};

#[test]
fn test_paragraph_rendering() {
    let backend = TestBackend::new(20, 3);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        let paragraph = Paragraph::new("Hello, World!");
        f.render_widget(paragraph, f.area());
    }).unwrap();

    let buffer = terminal.backend().buffer();
    assert_eq!(buffer.get(0, 0).symbol(), "H");
}
```

### Pros

- Fast (no I/O)
- Deterministic
- Easy to debug
- Can run in parallel
- No external dependencies

### Cons

- Doesn't test terminal integration
- Misses PTY-specific bugs
- Can't test graphics protocols
- Doesn't validate user experience

### Best For

- Widget libraries
- Layout engines
- Stateless components
- Business logic
- Regression prevention for specific components

## 2. Snapshot Testing

### What to Test

- Visual regression detection
- Layout changes across refactoring
- Widget appearance consistency
- Rendering output verification

### Tools

- `insta` crate (recommended)
- `expect-test` crate (lightweight alternative)
- Ratatui's `TestBackend` for rendering

### Example with insta

```rust
use ratatui::{backend::TestBackend, Terminal};
use insta::assert_snapshot;

#[test]
fn test_menu_layout() {
    let backend = TestBackend::new(40, 10);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|f| {
        render_menu(f);
    }).unwrap();

    assert_snapshot!(terminal.backend().buffer());
}
```

### Example with expect-test

```rust
use expect_test::expect;

#[test]
fn test_table_output() {
    let output = render_table_to_string();

    expect![[r#"
        ┌─────┬─────┐
        │ A   │ B   │
        ├─────┼─────┤
        │ 1   │ 2   │
        └─────┴─────┘
    "#]].assert_eq(&output);
}
```

### Pros

- Catches visual regressions
- Easy to review changes (cargo insta review)
- Documents expected output
- Faster than manual verification
- Scales to complex UIs

### Cons

- Snapshot files can be large
- Still uses TestBackend (inherits limitations)
- Can become brittle if output changes frequently
- Requires discipline to review snapshots

### Best For

- Regression testing
- Complex widget layouts
- Documenting expected output
- Refactoring with confidence
- Visual consistency validation

## 3. PTY-Based Integration Testing

### What to Test

- Full application behavior
- Terminal size negotiation
- Signal handling (SIGWINCH, SIGTERM, etc.)
- TTY detection logic
- Event loop integration
- User interaction flows
- Multi-step scenarios

### Tools

- `terminal-testlib` (this library)
- `portable-pty` (lower level)
- Custom test harnesses

### Example

```rust
use term_test::TuiTestHarness;
use std::process::Command;

#[test]
fn test_navigation_flow() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;

    // Spawn the application
    harness.spawn(Command::new("./target/debug/my-app"))?;

    // Wait for initial render
    harness.wait_for(|state| {
        state.contents().contains("Main Menu")
    })?;

    // Navigate down
    harness.send_key(Key::Down)?;
    harness.send_key(Key::Down)?;

    // Select item
    harness.send_key(Key::Enter)?;

    // Verify navigation worked
    harness.wait_for(|state| {
        state.contents().contains("Settings")
    })?;

    Ok(())
}
```

### Pros

- Tests real terminal environment
- Catches PTY-specific bugs
- Validates user experience
- Tests event handling
- Can test terminal resize
- Realistic integration testing

### Cons

- Slower than unit tests
- More complex setup
- May have timing issues
- Harder to debug
- Platform-specific behavior

### Best For

- Integration testing
- User flow validation
- PTY-specific features
- Terminal negotiation
- Event handling
- Real-world scenarios

## 4. Graphics Protocol Testing (Sixel)

### What to Test

- Sixel escape sequence correctness
- Image rendering integration
- Graphics protocol support detection
- Image dimension handling
- Color palette management

### Tools

- `terminal-testlib` with Sixel support
- Reference test images (libsixel, Jexer)
- PTY + terminal emulator with graphics support

### Example

```rust
use term_test::{TuiTestHarness, SixelCapture};

#[test]
fn test_image_display() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 40)?;
    harness.spawn(Command::new("./target/debug/image-viewer"))?;

    // Send command to display image
    harness.send_text("open test-image.png\n")?;

    harness.wait_for(|state| {
        state.contents().contains("Image loaded")
    })?;

    // Capture Sixel output
    let sixel = SixelCapture::from_screen(&harness.state)?;

    // Validate structure
    assert!(sixel.validate().is_ok());

    // Compare with known-good reference
    let reference = SixelCapture::from_file(
        "tests/fixtures/sixel/expected-output.six"
    )?;

    assert!(sixel.compare(&reference).is_ok());

    Ok(())
}
```

### Pros

- Validates graphics rendering
- Catches protocol errors
- Ensures compatibility
- Documents expected output

### Cons

- Complex to implement
- Requires graphics-capable terminal emulator
- Platform/terminal variations
- Large test fixtures

### Best For

- Graphics-heavy TUI applications
- Image viewers
- Chart/graph rendering
- Sixel/graphics protocol compliance
- Visual data display

## 5. Async/Event-Driven Testing

### What to Test

- Async event loops
- Concurrent operations
- Race conditions
- Timeout handling
- Background tasks
- Real-time updates

### Tools

- `tokio::test` or `async_std::test`
- `terminal-testlib` async harness
- Async test utilities

### Example

```rust
use term_test::AsyncTuiTestHarness;

#[tokio::test]
async fn test_real_time_updates() -> Result<()> {
    let mut harness = AsyncTuiTestHarness::new(80, 24)?;

    harness.spawn(Command::new("./target/debug/monitor-app")).await?;

    // Wait for initial state
    harness.wait_for(|state| {
        state.contents().contains("Monitoring...")
    }).await?;

    // Verify updates appear within timeout
    let result = tokio::time::timeout(
        Duration::from_secs(2),
        harness.wait_for(|state| {
            state.contents().contains("Update received")
        })
    ).await;

    assert!(result.is_ok());

    Ok(())
}
```

### Pros

- Tests async behavior
- Validates timing
- Catches race conditions
- Tests real event loops

### Cons

- More complex than sync tests
- Timing-sensitive
- May be flaky
- Harder to reproduce failures

### Best For

- Async Ratatui applications
- Real-time monitoring apps
- Network applications
- Background task coordination
- Event stream processing

## 6. Property-Based Testing

### What to Test

- Widget invariants
- Layout properties
- State machine correctness
- Input validation
- Edge cases

### Tools

- `proptest` crate
- `quickcheck` crate
- Custom generators

### Example

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_layout_always_fits(
        width in 10u16..200,
        height in 5u16..100
    ) {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();

        terminal.draw(|f| {
            render_responsive_layout(f);
        }).unwrap();

        // Verify no overflow
        let buffer = terminal.backend().buffer();
        for y in 0..height {
            for x in 0..width {
                assert!(buffer.get(x, y).is_some());
            }
        }
    }
}
```

### Pros

- Finds edge cases
- Tests invariants
- Explores input space
- Reduces manual test cases

### Cons

- Can be slow
- May find irrelevant failures
- Requires good generators
- Complex to set up

### Best For

- Layout engines
- Input parsers
- State machines
- Invariant validation
- Edge case discovery

## 7. Benchmark Testing

### What to Test

- Rendering performance
- Layout calculation speed
- Event processing throughput
- Memory usage
- Frame rate

### Tools

- `criterion` crate
- `divan` crate
- Custom benchmarks

### Example

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_complex_layout(c: &mut Criterion) {
    let backend = TestBackend::new(200, 100);
    let mut terminal = Terminal::new(backend).unwrap();

    c.bench_function("complex_layout", |b| {
        b.iter(|| {
            terminal.draw(|f| {
                render_complex_layout(black_box(f));
            }).unwrap();
        });
    });
}

criterion_group!(benches, bench_complex_layout);
criterion_main!(benches);
```

### Pros

- Quantifies performance
- Tracks regressions
- Guides optimization
- Documents performance characteristics

### Cons

- Doesn't test correctness
- Platform-dependent results
- Requires baseline establishment
- Can be time-consuming

### Best For

- Performance-critical widgets
- Large data rendering
- Optimization validation
- Performance regression detection

## Testing Strategy Recommendations

### For Widget Libraries

```
70% Unit tests (TestBackend)
20% Snapshot tests (insta)
10% Property-based tests (proptest)
```

### For Complete TUI Applications

```
40% Unit tests (TestBackend)
30% Integration tests (terminal-testlib PTY)
20% Snapshot tests (insta)
10% E2E scenarios (terminal-testlib)
```

### For Graphics-Heavy Applications

```
30% Unit tests (TestBackend)
30% Integration tests (terminal-testlib PTY)
25% Graphics protocol tests (Sixel)
15% Snapshot tests (insta)
```

### For Real-Time/Async Applications

```
35% Unit tests (TestBackend)
35% Async integration tests (terminal-testlib async)
20% Event-driven tests
10% Snapshot tests (insta)
```

## Common Testing Patterns

### Pattern 1: Test-Driven Development (TDD)

```rust
// 1. Write failing test
#[test]
fn test_menu_selection() {
    let mut app = App::new();
    app.select_next();
    assert_eq!(app.selected(), 1);
}

// 2. Implement minimum to pass
impl App {
    fn select_next(&mut self) {
        self.selected += 1;
    }
}

// 3. Refactor
// 4. Repeat
```

### Pattern 2: Snapshot-Driven Development

```rust
// 1. Implement feature
fn render_dashboard(f: &mut Frame) {
    // Implementation
}

// 2. Create snapshot test
#[test]
fn test_dashboard() {
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(render_dashboard).unwrap();
    assert_snapshot!(terminal.backend().buffer());
}

// 3. Review and accept snapshot
// 4. Future changes compared against snapshot
```

### Pattern 3: Behavior-Driven Development (BDD)

```rust
#[test]
fn test_user_can_navigate_menu() {
    // Given: Application is started
    let mut harness = TuiTestHarness::new(80, 24).unwrap();
    harness.spawn(app_command()).unwrap();

    // When: User navigates down twice
    harness.send_key(Key::Down).unwrap();
    harness.send_key(Key::Down).unwrap();

    // Then: Third item is selected
    harness.wait_for(|state| {
        state.contents().contains("> Item 3")
    }).unwrap();
}
```

### Pattern 4: Arrange-Act-Assert (AAA)

```rust
#[test]
fn test_widget_styling() {
    // Arrange
    let widget = MyWidget::new()
        .style(Style::default().fg(Color::Red));
    let backend = TestBackend::new(10, 1);
    let mut terminal = Terminal::new(backend).unwrap();

    // Act
    terminal.draw(|f| {
        f.render_widget(widget, f.area());
    }).unwrap();

    // Assert
    let cell = terminal.backend().buffer().get(0, 0);
    assert_eq!(cell.fg, Color::Red);
}
```

## Test Organization

### Recommended Structure

```
my-tui-app/
├── src/
│   ├── lib.rs
│   ├── widgets/
│   │   ├── mod.rs
│   │   ├── menu.rs       # Widget implementation
│   │   └── menu_tests.rs # Unit tests alongside code
│   └── app.rs
├── tests/
│   ├── integration/
│   │   ├── navigation.rs    # User flow tests
│   │   ├── rendering.rs     # Full render tests
│   │   └── async_events.rs  # Async behavior tests
│   ├── fixtures/
│   │   ├── sixel/
│   │   │   └── test-image.six
│   │   └── data/
│   │       └── sample-data.json
│   └── snapshots/          # Insta snapshots
│       └── *.snap
├── benches/
│   └── rendering.rs        # Performance benchmarks
└── examples/
    └── manual_test.rs      # For manual verification
```

### Naming Conventions

```rust
// Unit tests
#[test]
fn test_widget_renders_correctly() { }

#[test]
fn test_layout_calculation_when_width_exceeds_height() { }

// Integration tests
#[test]
fn test_user_can_navigate_and_select() { }

#[test]
fn test_app_handles_terminal_resize() { }

// Benchmark tests
fn bench_complex_widget_rendering(c: &mut Criterion) { }
```

## Debugging Failed Tests

### Snapshot Test Failures

```bash
# Review snapshot differences
cargo insta review

# Update all snapshots
cargo insta test --review

# Only review for specific test
cargo insta test --review test_name
```

### PTY Test Failures

```rust
// Add debug output
#[test]
fn test_with_debug() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;
    harness.spawn(Command::new("./my-app"))?;

    // Print current screen state
    eprintln!("Screen contents:\n{}", harness.screen_contents());

    // Take snapshot for debugging
    std::fs::write(
        "/tmp/test-output.txt",
        harness.screen_contents()
    )?;

    Ok(())
}
```

### Timing Issues

```rust
// Increase timeout
harness.set_timeout(Duration::from_secs(10))?;

// Add explicit waits
harness.wait_for(|state| {
    eprintln!("Waiting, current: {}", state.contents());
    state.contents().contains("Expected")
})?;
```

## Continuous Integration

### GitHub Actions Example

```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run unit tests
        run: cargo test --lib

      - name: Run integration tests
        run: cargo test --test '*'

      - name: Run benchmarks (check only)
        run: cargo bench --no-run
```

## References

- [Rust Testing Best Practices](https://rust-exercises.com/advanced-testing/)
- [Ratatui Testing Recipes](https://ratatui.rs/recipes/testing/snapshots/)
- [Testing TUI Apps Blog](https://blog.waleedkhan.name/testing-tui-apps/)
- [Property-Based Testing in Rust](https://proptest-rs.github.io/proptest/)

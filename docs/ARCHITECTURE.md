# terminal-testlib Architecture

## Purpose

`terminal-testlib` is a Rust library for integration testing of terminal user interface (TUI) applications, particularly those built with Ratatui. It enables testing of features that require actual terminal escape sequence processing, including:

- ANSI/VT escape sequences
- Color and styling
- Sixel image rendering
- Mouse events
- Terminal resize handling
- Complex rendering scenarios

## Design Principles

1. **Ease of Use**: Simple, ergonomic API for common testing patterns
2. **Completeness**: Support for testing all terminal features, including graphics protocols
3. **Cross-Platform**: Work consistently across Linux, macOS, and Windows (MVP: Linux focus)
4. **Snapshot-Friendly**: Natural integration with snapshot testing frameworks
5. **Async-First**: Support for async Ratatui applications (Tokio for MVP)
6. **Bevy-Ready**: First-class integration with Bevy ECS and bevy_ratatui
7. **Minimal Dependencies**: Reuse well-maintained components from the ecosystem

## MVP Focus

**Primary Use Case**: Enable comprehensive integration testing for the [dgx-pixels](https://github.com/raibid-labs/dgx-pixels) project, a Bevy-based TUI application with Sixel graphics support.

**Key Requirements**:
- Sixel graphics position verification and bounds checking
- Bevy ECS integration (query entities, control update cycles)
- bevy_ratatui plugin support
- Tokio async runtime
- Headless CI/CD compatibility

## Architecture Layers

### Layer 1: PTY Management

**Purpose**: Create and manage pseudo-terminal (PTY) for running TUI applications

**Implementation**: `portable-pty` crate (from WezTerm)

**Responsibilities**:
- Create PTY with configurable size
- Spawn processes in PTY
- Read/write to PTY
- Handle cross-platform differences

**Key Types**:
```rust
pub struct TestTerminal {
    pty_pair: PtyPair,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
    child: Option<Child>,
}
```

### Layer 2: Terminal Emulation

**Purpose**: Parse terminal escape sequences and maintain screen state

**Implementation**: `vt100` crate

**Responsibilities**:
- Parse ANSI/VT escape sequences
- Maintain screen buffer
- Track cursor position, colors, attributes
- Support Sixel and other graphics protocols (if available)

**Key Types**:
```rust
pub struct ScreenState {
    parser: vt100::Parser,
    width: u16,
    height: u16,
}

impl ScreenState {
    pub fn feed(&mut self, data: &[u8]);
    pub fn screen(&self) -> &vt100::Screen;
    pub fn contents(&self) -> String;
    pub fn cell(&self, row: u16, col: u16) -> Cell;
}
```

### Layer 3: Test Harness

**Purpose**: Coordinate PTY and terminal emulation, provide high-level testing API

**Responsibilities**:
- Spawn TUI application in PTY
- Feed PTY output to terminal emulator
- Send input events (keyboard, mouse, resize)
- Capture screen state at any point
- Wait for specific screen conditions
- Integration with snapshot testing

**Key Types**:
```rust
pub struct TuiTestHarness {
    terminal: TestTerminal,
    state: ScreenState,
    timeout: Duration,
}

impl TuiTestHarness {
    pub fn new(width: u16, height: u16) -> Result<Self>;
    pub fn spawn(&mut self, cmd: Command) -> Result<()>;
    pub fn send_key(&mut self, key: Key) -> Result<()>;
    pub fn send_text(&mut self, text: &str) -> Result<()>;
    pub fn send_mouse(&mut self, event: MouseEvent) -> Result<()>;
    pub fn resize(&mut self, width: u16, height: u16) -> Result<()>;
    pub fn wait_for(&mut self, condition: impl Fn(&ScreenState) -> bool) -> Result<()>;
    pub fn snapshot(&self) -> Snapshot;
    pub fn screen_contents(&self) -> String;
}
```

### Layer 4: Snapshot Testing Integration

**Purpose**: Provide ergonomic snapshot testing capabilities

**Implementation**: Integration with `expect-test` and/or `insta`

**Responsibilities**:
- Serialize screen state for comparison
- Support multiple snapshot formats (text, JSON, etc.)
- Diff visualization for test failures
- Auto-update capabilities

**Key Types**:
```rust
pub struct Snapshot {
    contents: String,
    metadata: SnapshotMetadata,
}

pub struct SnapshotMetadata {
    width: u16,
    height: u16,
    cursor_pos: (u16, u16),
    timestamp: SystemTime,
}

impl Snapshot {
    pub fn as_text(&self) -> String;
    pub fn as_json(&self) -> serde_json::Value;
    pub fn compare(&self, other: &Snapshot) -> Diff;
}
```

### Layer 5: Ratatui Integration (Optional Helpers)

**Purpose**: Provide Ratatui-specific testing utilities

**Responsibilities**:
- Helpers for common Ratatui patterns
- Widget-specific assertions
- Layout verification
- Event simulation
- High-level assertions (text_at, cursor_in_area, etc.)

**Key Types**:
```rust
pub struct RatatuiTestHelper {
    harness: TuiTestHarness,
}

impl RatatuiTestHelper {
    // Widget assertions
    pub fn assert_widget_at(&self, x: u16, y: u16, expected: &str) -> Result<()>;
    pub fn assert_layout(&self, expected_layout: &Layout) -> Result<()>;

    // Text assertions (MVP)
    pub fn assert_text_at(&self, x: u16, y: u16, text: &str) -> Result<()>;
    pub fn assert_text_contains(&self, text: &str) -> Result<()>;
    pub fn assert_area_contains_text(&self, area: Rect, text: &str) -> Result<()>;

    // Cursor assertions (MVP)
    pub fn assert_cursor_position(&self, x: u16, y: u16) -> Result<()>;
    pub fn assert_cursor_in_area(&self, area: Rect) -> Result<()>;
    pub fn get_cursor_position(&self) -> (u16, u16);

    // Event simulation
    pub fn send_event(&mut self, event: crossterm::event::Event) -> Result<()>;
}
```

### Layer 6: Bevy ECS Integration (MVP Requirement)

**Purpose**: Support testing of Bevy-based TUI applications using bevy_ratatui

**Implementation**: Wrapper around TuiTestHarness + Bevy App

**Responsibilities**:
- Initialize headless Bevy app for testing
- Control Bevy update cycles frame-by-frame
- Query entities and components
- Access resources
- Send Bevy events
- Coordinate terminal rendering with Bevy systems

**Key Types**:
```rust
#[cfg(feature = "bevy")]
pub struct BevyTuiTestHarness {
    harness: TuiTestHarness,
    app: bevy::app::App,
}

#[cfg(feature = "bevy")]
impl BevyTuiTestHarness {
    // Initialization
    pub fn new() -> Result<Self>;
    pub fn with_plugins(plugins: impl Plugins) -> Result<Self>;
    pub fn with_bevy_ratatui() -> Result<Self>;  // Convenience for bevy_ratatui

    // Bevy update control (MVP requirement)
    pub fn update(&mut self) -> Result<()>;  // Run one Bevy frame
    pub fn update_n(&mut self, count: usize) -> Result<()>;  // Run N frames
    pub fn render_frame(&mut self) -> Result<()>;  // Update + render to terminal

    // ECS querying (MVP requirement)
    pub fn query<T: Component>(&self) -> Vec<&T>;
    pub fn query_mut<T: Component>(&mut self) -> Vec<&mut T>;
    pub fn get_resource<T: Resource>(&self) -> Option<&T>;
    pub fn get_resource_mut<T: Resource>(&mut self) -> Option<&mut T>;
    pub fn world(&self) -> &World;  // Direct World access for complex queries

    // Event integration
    pub fn send_bevy_event<T: Event>(&mut self, event: T);
    pub fn read_bevy_events<T: Event>(&self) -> Vec<&T>;

    // Terminal operations (delegates to inner harness)
    pub fn press_key(&mut self, key: KeyCode) -> Result<()>;
    pub fn type_text(&mut self, text: &str) -> Result<()>;
    pub fn wait_for(&mut self, condition: impl Fn(&ScreenState) -> bool) -> Result<()>;
    pub fn snapshot(&self) -> Snapshot;

    // Sixel assertions (MVP requirement)
    pub fn has_sixel_graphics(&self) -> bool;
    pub fn capture_sixel_state(&self) -> SixelCapture;
    pub fn assert_sixel_within(&self, area: Rect) -> Result<()>;
    pub fn assert_no_sixel_outside(&self, area: Rect) -> Result<()>;

    // High-level assertions (dgx-pixels use case)
    pub fn assert_on_screen(&self, screen: impl ScreenMarker) -> Result<()>;
}
```

## Sixel Testing Support

### Challenge

Sixel rendering produces complex escape sequences that represent images. Testing requires:
1. Verifying the escape sequence structure is correct
2. **Verifying position (cursor location when rendered)** - MVP requirement
3. **Verifying bounds (stays within designated area)** - MVP requirement
4. Optionally verifying the rendered image looks correct

### Solution

**Level 1: Sequence Verification with Position Tracking** (MVP)
- Capture raw Sixel escape sequences from PTY
- **Track cursor position when each Sixel is rendered** (critical!)
- Parse and validate structure
- Calculate bounds (position + dimensions)
- Support area-bounded queries (in/outside area)

**Level 2: Clearing Detection** (MVP)
- Capture Sixel state before/after screen transitions
- Detect when Sixel graphics are cleared
- Compare snapshots to verify clearing

**Level 3: Visual Verification** (Post-MVP)
- Decode Sixel to image data
- Compare against expected image (pixel-by-pixel or perceptual hash)
- Integration with image testing libraries

**Implementation**:
```rust
pub struct SixelCapture {
    sequences: Vec<SixelSequence>,
}

pub struct SixelSequence {
    raw: Vec<u8>,
    position: (u16, u16),  // MVP: Cursor position when rendered
    bounds: Rect,          // MVP: Calculated from position + dimensions
    width: u32,
    height: u32,
    colors: Vec<Color>,
}

impl SixelCapture {
    pub fn from_screen(state: &ScreenState) -> Self;
    pub fn validate(&self) -> Result<()>;
    pub fn compare(&self, expected: &SixelCapture) -> Result<()>;

    // MVP: Position-based queries
    pub fn is_empty(&self) -> bool;
    pub fn sequences_in_area(&self, area: Rect) -> Vec<&SixelSequence>;
    pub fn sequences_outside_area(&self, area: Rect) -> Vec<&SixelSequence>;
    pub fn assert_all_within(&self, area: Rect) -> Result<()>;
    pub fn differs_from(&self, other: &SixelCapture) -> bool;
}
```

**Test Image Repository**:
```
tests/
  fixtures/
    sixel/
      snake.six          # From libsixel
      map8.six           # From libsixel
      jexer/             # From Jexer test suite
        *.six
        *.png            # Expected rendering
```

## Async Support

### Challenge

Modern Ratatui apps often use async runtime (Tokio, async-std)

### Solution

**Option 1: Test-Runtime Agnostic**
```rust
pub struct AsyncTuiTestHarness {
    harness: TuiTestHarness,
}

impl AsyncTuiTestHarness {
    pub async fn send_key(&mut self, key: Key) -> Result<()>;
    pub async fn wait_for(&mut self, condition: impl Fn(&ScreenState) -> bool) -> Result<()>;
}
```

**Option 2: Runtime-Specific Helpers**
```rust
#[cfg(feature = "tokio")]
pub mod tokio {
    pub struct TokioTuiTestHarness { ... }
}

#[cfg(feature = "async-std")]
pub mod async_std {
    pub struct AsyncStdTuiTestHarness { ... }
}
```

## Module Structure

```
terminal-testlib/
├── src/
│   ├── lib.rs              # Public API exports
│   ├── pty.rs              # PTY management (Layer 1)
│   ├── screen.rs           # Terminal emulation (Layer 2)
│   ├── harness.rs          # Test harness (Layer 3)
│   ├── snapshot.rs         # Snapshot testing (Layer 4)
│   ├── ratatui.rs          # Ratatui helpers (Layer 5)
│   ├── bevy.rs             # Bevy ECS integration (Layer 6) - MVP
│   ├── sixel.rs            # Sixel support with position tracking - MVP
│   ├── async_support.rs    # Async helpers (Tokio for MVP)
│   └── util.rs             # Utilities
├── tests/
│   ├── integration/        # Integration tests
│   ├── fixtures/           # Test fixtures (Sixel images, etc.)
│   └── examples/           # Example TUI apps for testing
├── examples/               # Usage examples
│   ├── basic_test.rs
│   ├── snapshot_test.rs
│   ├── sixel_test.rs
│   └── async_test.rs
└── benches/                # Benchmarks
    └── parsing.rs
```

## Example Usage

### Basic Test

```rust
use term_test::TuiTestHarness;

#[test]
fn test_hello_world_tui() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;

    // Spawn TUI app
    harness.spawn(Command::new("./target/debug/my-tui-app"))?;

    // Wait for initial render
    harness.wait_for(|state| {
        state.contents().contains("Welcome")
    })?;

    // Send keypress
    harness.send_key(Key::Down)?;

    // Verify state changed
    let snapshot = harness.snapshot();
    assert!(snapshot.contents().contains("Selected: Item 2"));

    Ok(())
}
```

### Snapshot Test

```rust
use term_test::TuiTestHarness;
use expect_test::expect;

#[test]
fn test_menu_rendering() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;
    harness.spawn(Command::new("./target/debug/my-tui-app"))?;

    harness.wait_for(|state| state.contents().contains("Menu"))?;

    let snapshot = harness.snapshot();

    expect![[r#"
        ┌─ Menu ─────────────────┐
        │ > Item 1               │
        │   Item 2               │
        │   Item 3               │
        └────────────────────────┘
    "#]].assert_eq(&snapshot.as_text());

    Ok(())
}
```

### Sixel Test

```rust
use term_test::{TuiTestHarness, SixelCapture};

#[test]
fn test_image_rendering() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;
    harness.spawn(Command::new("./target/debug/image-viewer"))?;

    // Wait for image to render
    harness.wait_for(|state| {
        state.contents().contains("Image loaded")
    })?;

    // Capture Sixel sequences
    let sixel = SixelCapture::from_screen(&harness.state);

    // Validate structure
    sixel.validate()?;

    // Compare with expected (snapshot)
    let expected = SixelCapture::from_file("tests/fixtures/sixel/expected.six")?;
    sixel.compare(&expected)?;

    Ok(())
}
```

### Async Test

```rust
use term_test::AsyncTuiTestHarness;

#[tokio::test]
async fn test_async_tui() -> Result<()> {
    let mut harness = AsyncTuiTestHarness::new(80, 24)?;

    harness.spawn(Command::new("./target/debug/async-tui-app")).await?;

    harness.wait_for(|state| {
        state.contents().contains("Ready")
    }).await?;

    harness.send_key(Key::Enter).await?;

    // Wait with timeout
    tokio::time::timeout(
        Duration::from_secs(5),
        harness.wait_for(|state| state.contents().contains("Complete"))
    ).await??;

    Ok(())
}
```

### Bevy TUI Test (dgx-pixels MVP Use Case)

```rust
use term_test::BevyTuiTestHarness;
use dgx_pixels::{Job, JobStatus, Screen};

#[tokio::test]
async fn test_sixel_renders_in_preview_area() -> Result<()> {
    // Create Bevy TUI test harness
    let mut test = BevyTuiTestHarness::with_bevy_ratatui()?;

    // Setup: Load test image
    test.load_test_image("tests/fixtures/test-sprite.png")?;

    // Navigate to Gallery screen (Tab or '2')
    test.press_key(KeyCode::Char('2'))?;
    test.update()?;  // Process navigation event
    test.render_frame()?;

    // Get preview area from component
    let preview_panel = test.query::<PreviewPanel>().first().unwrap();
    let preview_area = preview_panel.area;

    // Assert: Sixel graphics are within preview area
    test.assert_sixel_within(preview_area)?;
    test.assert_no_sixel_outside(preview_area)?;

    // Assert: On correct screen
    test.assert_on_screen(Screen::Gallery)?;

    Ok(())
}

#[tokio::test]
async fn test_sixel_clears_on_screen_change() -> Result<()> {
    let mut test = BevyTuiTestHarness::with_bevy_ratatui()?;

    // Setup: Render image on Gallery screen
    test.load_test_image("tests/fixtures/test-sprite.png")?;
    test.press_key(KeyCode::Char('2'))?;  // Gallery
    test.update()?;
    test.render_frame()?;

    // Verify image is rendered
    assert!(test.has_sixel_graphics());
    let sixel_before = test.capture_sixel_state()?;

    // Navigate to different screen
    test.press_key(KeyCode::Char('1'))?;  // Generation screen
    test.update()?;
    test.render_frame()?;

    // Assert: Sixel graphics are cleared
    assert!(!test.has_sixel_graphics());
    test.assert_on_screen(Screen::Generation)?;

    // Verify state changed
    let sixel_after = test.capture_sixel_state()?;
    assert!(sixel_after.differs_from(&sixel_before));

    Ok(())
}

#[tokio::test]
async fn test_job_submission_creates_entity() -> Result<()> {
    let mut test = BevyTuiTestHarness::with_bevy_ratatui()?;

    // Navigate to Generation screen
    test.press_key(KeyCode::Char('1'))?;
    test.update()?;

    // Type a prompt
    test.type_text("pixel art sword")?;
    test.update()?;

    // Submit job
    test.press_key(KeyCode::Enter)?;
    test.update_n(2)?;  // Process input + event handler systems

    // Query Bevy World for Job entity
    let jobs = test.query::<Job>();
    assert_eq!(jobs.len(), 1);

    let job = jobs.first().unwrap();
    assert_eq!(job.prompt, "pixel art sword");
    assert_eq!(job.status, JobStatus::Pending);

    Ok(())
}
```

## Dependencies

### Core Dependencies
- `portable-pty` - PTY management
- `vt100` - Terminal emulation
- `thiserror` - Error handling
- `serde` - Serialization (optional, for JSON snapshots)

### MVP Dependencies
- `tokio` - Async runtime support (feature: "async-tokio") **MVP**
- `bevy` - Bevy ECS integration (feature: "bevy") **MVP**
- `bevy_ecs` - Bevy ECS (feature: "bevy") **MVP**
- `bevy_ratatui` - bevy_ratatui plugin support (feature: "bevy-ratatui") **MVP**
- `ratatui` - Ratatui helpers (feature: "ratatui-helpers") **MVP**
- `crossterm` - Event types (feature: "ratatui-helpers") **MVP**
- `insta` - Snapshot testing (feature: "snapshot-insta") **MVP**
- `serde` - Serialization for snapshots (feature: "snapshot-insta") **MVP**
- `serde_json` - JSON serialization (feature: "snapshot-insta") **MVP**

### Post-MVP Dependencies
- `async-std` - Async runtime support (feature: "async-async-std")
- `expect-test` - Snapshot testing (feature: "snapshot-expect")
- `image` - Sixel image comparison (feature: "sixel-image")

## Feature Flags

```toml
[features]
default = []

# MVP features
async-tokio = ["tokio"]
bevy = ["dep:bevy", "bevy_ecs"]
bevy-ratatui = ["bevy", "dep:bevy_ratatui"]
ratatui-helpers = ["ratatui", "crossterm"]
sixel = []  # Core Sixel support with position tracking
snapshot-insta = ["insta", "serde", "serde_json"]

# MVP bundle
mvp = [
    "async-tokio",
    "bevy",
    "bevy-ratatui",
    "ratatui-helpers",
    "sixel",
    "snapshot-insta",
]

# Post-MVP features
async-async-std = ["async-std"]
snapshot-expect = ["expect-test"]
sixel-image = ["image"]  # Advanced Sixel decoding

# Full bundle (all features)
full = [
    "mvp",
    "async-async-std",
    "snapshot-expect",
    "sixel-image",
]
```

## Error Handling

```rust
#[derive(Debug, thiserror::Error)]
pub enum TermTestError {
    #[error("PTY error: {0}")]
    Pty(#[from] portable_pty::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Timeout waiting for condition")]
    Timeout,

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Snapshot mismatch: {0}")]
    SnapshotMismatch(String),

    #[error("Sixel validation failed: {0}")]
    SixelValidation(String),
}

pub type Result<T> = std::result::Result<T, TermTestError>;
```

## Performance Considerations

1. **Buffering**: Efficient buffering of PTY output to minimize syscalls
2. **Parsing**: Leverage vt100's efficient parsing (reuse allocations)
3. **Snapshot Size**: Option to capture only changed regions
4. **Async I/O**: Non-blocking I/O for async harness
5. **Test Fixtures**: Lazy loading of test images

## Future Enhancements

1. **Record/Replay**: Record terminal sessions and replay for testing
2. **Visual Regression**: Compare screenshots (requires actual rendering)
3. **Fuzz Testing**: Generate random input sequences to find crashes
4. **Coverage**: Terminal coverage (which parts of screen were tested)
5. **Multi-Terminal**: Test split-pane and multiplexer scenarios
6. **Remote Testing**: Test over SSH or other remote protocols
7. **Performance Profiling**: Built-in profiling of TUI app performance

## Alternative Approaches Considered

### Using WezTerm Directly

**Pros**:
- Complete terminal emulator
- Full Sixel support
- Active development

**Cons**:
- Heavy dependency
- Requires GUI backend (even if headless)
- Complex integration

**Decision**: Use WezTerm's components (portable-pty, termwiz) but not entire emulator

### Custom Terminal Emulator

**Pros**:
- Full control
- Optimized for testing

**Cons**:
- Significant development effort
- Maintenance burden
- Reinventing wheel

**Decision**: Use existing vt100 crate

### Headless Backend for Ratatui

**Pros**:
- No PTY needed
- Deterministic
- Fast

**Cons**:
- Doesn't test actual terminal integration
- Misses PTY-specific issues
- Ratatui doesn't provide official headless backend

**Decision**: PTY-based testing is more realistic, but could add headless option later

## Comparison with Existing Solutions

### vs. Ratatui's TestBackend

**Ratatui TestBackend**:
- Unit testing of widgets
- No PTY
- Text-based assertions

**terminal-testlib**:
- Integration testing of full TUI apps
- Real PTY
- Supports Sixel and graphics
- Snapshot testing

**Conclusion**: Complementary, not competitive. Use TestBackend for unit tests, terminal-testlib for integration tests.

### vs. Integration Tests in Other Languages

**expect (Tcl)**:
- Script-based
- Interactive program automation
- Limited to text

**Python pexpect**:
- Similar to expect
- Text-based assertions

**terminal-testlib**:
- Rust-native
- Type-safe
- Graphics support
- Modern snapshot testing

## Success Criteria

The library will be considered successful if:

1. **Easy to adopt**: Existing Ratatui projects can add integration tests in < 10 lines of code
2. **Comprehensive**: Supports testing all terminal features including Sixel
3. **Reliable**: Tests are deterministic and don't flake
4. **Fast**: Test execution is fast enough for CI/CD
5. **Well-documented**: Examples cover common use cases
6. **Maintained**: Dependencies are actively maintained and cross-platform

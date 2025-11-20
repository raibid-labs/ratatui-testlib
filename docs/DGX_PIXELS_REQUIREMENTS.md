# dgx-pixels Integration Requirements Analysis

## Overview

This document analyzes how the term-test library design addresses the specific requirements from the dgx-pixels project (Issue #1), identifies gaps, and proposes adjustments to the roadmap.

## Requirements Coverage

### ✅ Well Covered by Current Design

#### 1. Terminal Emulator Support

**Requirement**: Headless terminal emulator for CI/CD with ANSI, Sixel, CSI support

**Current Design**:
- ✅ Layer 1 (PTY Management) + Layer 2 (Terminal Emulation)
- ✅ Using vt100 crate for escape sequence parsing
- ✅ Configurable terminal size in `TuiTestHarness::new(width, height)`
- ✅ Screen state inspection via `ScreenState`

**Phase**: Phase 1 (Core PTY Harness)

#### 2. Ratatui Integration

**Requirement**: Compatible with ratatui 0.29.x

**Current Design**:
- ✅ Layer 5 (Ratatui Integration Helpers)
- ✅ Event simulation in Phase 2
- ✅ Frame-by-frame rendering via PTY

**Phase**: Phase 6 (Ratatui Helpers)

**Note**: Version compatibility needs to be explicitly tested

#### 3. Sixel Graphics Testing

**Requirement**: Parse sequences, verify position, validate clearing, bounds checking

**Current Design**:
- ✅ Phase 5 (Sixel & Graphics Support)
- ✅ `SixelCapture` type for sequence parsing
- ✅ Validation and comparison methods

**Phase**: Phase 5

**Gap**: Current design focuses on sequence validation but may need enhancement for position verification

#### 4. Screen State Inspection

**Requirement**: Snapshots, coordinate validation, content extraction

**Current Design**:
- ✅ Phase 3 (Snapshot Integration)
- ✅ `Snapshot` type with metadata
- ✅ `ScreenState` with cell queries

**Phase**: Phase 3

#### 5. Event Simulation

**Requirement**: Keyboard events, timing control, event sequences

**Current Design**:
- ✅ Phase 2 (Event Simulation & Conditions)
- ✅ `send_key()`, `send_text()`, `send_mouse()`
- ✅ Smart waiting with conditions

**Phase**: Phase 2

### ⚠️ Partially Covered

#### 6. Async Runtime Support

**Requirement**: Tokio + Bevy ECS integration

**Current Design**:
- ✅ Phase 4 (Async Support) - Tokio/async-std
- ⚠️ **Gap**: No specific Bevy ECS integration planned
- ⚠️ **Gap**: No frame-by-frame control for Bevy update cycles

**Recommendation**: Add Bevy-specific helpers to Phase 6

### ❌ Not Currently Covered

#### 7. Bevy ECS-Specific Testing

**Requirement**: Query entities, test Bevy systems, control update cycles

**Current Design**: Not addressed

**Impact**: **HIGH** - This is critical for dgx-pixels

**Recommendation**: Add new Phase 6.5 or extend Phase 6

#### 8. Performance Benchmarking

**Requirement**: Frame time, latency, memory tracking

**Current Design**: Mentioned in Future Enhancements but not planned for 1.0

**Recommendation**: Consider adding basic profiling to Phase 7 (Polish)

#### 9. Interactive Debugging

**Requirement**: Session recording, failed test screenshots, verbose logging

**Current Design**: Mentioned in Future Enhancements (Record/Replay)

**Recommendation**: Add basic debug output to Phase 7

## API Design Comparison

### dgx-pixels Desired API

```rust
// From Issue #1
test.assert_sixel_at(area, expected_data)?;
test.assert_sixel_cleared(area)?;
test.assert_no_sixel_outside(area)?;
test.assert_text_at(10, 5, "Generation")?;
test.assert_area_contains_text(preview_area, "1024x1024")?;
test.on_screen(Screen::Gallery)?;
```

### Current term-test Design

```rust
// From ARCHITECTURE.md
harness.wait_for(|state| state.contents().contains("Generation"))?;
let sixel = SixelCapture::from_screen(&harness.state)?;
sixel.validate()?;
```

### Analysis

**Strengths of dgx-pixels API**:
- More declarative and readable
- Coordinate-based assertions
- Area-bounded checks
- Domain-specific (screen, area)

**Current Design Strengths**:
- More flexible (condition-based waiting)
- Lower-level control

**Recommendation**: Add both!
- Keep low-level API for flexibility
- Add high-level assertion helpers in Phase 6

## Specific Use Case Analysis

### Use Case 1: Sixel Preview Positioning

**dgx-pixels needs**:
```rust
tui.assert_sixel_within_bounds(preview_area)?;
tui.assert_no_sixel_outside(preview_area)?;
tui.assert_last_sixel_position(row, col)?;
```

**Current design provides**:
```rust
let sixel = SixelCapture::from_screen(&harness.state)?;
// Need to add position checking
```

**Gap**: Need to extract Sixel **position** from terminal state, not just sequence data

**Solution**: Enhance `SixelCapture` to include position metadata:

```rust
pub struct SixelSequence {
    raw: Vec<u8>,
    position: (u16, u16),  // ADD THIS
    width: u32,
    height: u32,
    colors: Vec<Color>,
}

impl SixelCapture {
    pub fn sequences_in_area(&self, area: Rect) -> Vec<&SixelSequence>;
    pub fn sequences_outside_area(&self, area: Rect) -> Vec<&SixelSequence>;
    pub fn assert_all_within(&self, area: Rect) -> Result<()>;
}
```

### Use Case 2: Screen Transition Cleanup

**dgx-pixels needs**:
```rust
tui.assert_has_sixel_graphics()?;
tui.capture_sixel_state()?;
tui.assert_no_sixel_graphics()?;
tui.assert_sixel_state_differs(snapshot)?;
```

**Current design provides**:
```rust
let before = harness.snapshot();
// ... perform action ...
let after = harness.snapshot();
// Need to compare sixel content specifically
```

**Gap**: Need Sixel-specific snapshot comparison

**Solution**: Add methods to `SixelCapture`:

```rust
impl SixelCapture {
    pub fn is_empty(&self) -> bool;
    pub fn differs_from(&self, other: &SixelCapture) -> bool;
}

impl TuiTestHarness {
    pub fn has_sixel_graphics(&self) -> bool;
    pub fn capture_sixel_state(&self) -> SixelCapture;
}
```

### Use Case 3: Text Input Processing

**dgx-pixels needs**:
```rust
tui.type_text("pixel art sword")?;
tui.assert_text_contains("pixel art sword")?;
tui.assert_cursor_in_area(input_area)?;
```

**Current design provides**:
```rust
harness.send_text("pixel art sword")?;
harness.wait_for(|state| state.contents().contains("pixel art sword"))?;
```

**Gap**: Cursor position and area-based assertions

**Solution**: Add to Phase 6 (Ratatui Helpers):

```rust
impl RatatuiTestHelper {
    pub fn assert_text_contains(&self, text: &str) -> Result<()>;
    pub fn assert_cursor_in_area(&self, area: Rect) -> Result<()>;
    pub fn get_cursor_position(&self) -> (u16, u16);
}
```

### Use Case 4: Bevy ECS Integration

**dgx-pixels needs**:
```rust
tui.update(); // Process Bevy update cycles
let jobs = tui.query_entities::<Job>()?;
```

**Current design provides**: **Nothing** ❌

**Gap**: This is a **major gap** - no Bevy integration planned

**Solution**: Add new component to Phase 6:

```rust
// NEW: Bevy-specific test harness
pub struct BevyTuiTestHarness {
    harness: TuiTestHarness,
    app: App,  // Bevy app instance
}

impl BevyTuiTestHarness {
    pub fn new() -> Result<Self>;
    pub fn update(&mut self) -> Result<()>;  // Run one Bevy update
    pub fn update_n(&mut self, n: usize) -> Result<()>;  // Run N updates
    pub fn query_entities<T: Component>(&self) -> Vec<&T>;
    pub fn get_resource<T: Resource>(&self) -> Option<&T>;
    pub fn send_bevy_event<T: Event>(&mut self, event: T);
}
```

## Critical Gaps Summary

| Gap | Priority | Impact | Recommendation |
|-----|----------|--------|----------------|
| **Sixel position tracking** | HIGH | Can't verify positioning bugs | Enhance Phase 5 |
| **Bevy ECS integration** | HIGH | Can't test dgx-pixels | Add to Phase 6 |
| **Area-based assertions** | MEDIUM | Less ergonomic API | Add to Phase 6 |
| **Cursor position queries** | MEDIUM | Can't verify text input | Add to Phase 6 |
| **Performance profiling** | LOW | Nice to have | Phase 7 or Future |
| **Debug session recording** | LOW | Debugging aid | Future |

## Roadmap Adjustments

### Phase 5 Enhancements: Sixel Position Tracking

**Add to existing Phase 5 tasks**:

- [ ] Track Sixel sequence positions (cursor location when drawn)
- [ ] Implement area-bounded Sixel queries
- [ ] Add position-based assertions
- [ ] Support Sixel clearing verification

**New Types**:

```rust
pub struct SixelSequence {
    raw: Vec<u8>,
    position: (u16, u16),      // NEW
    bounds: Rect,              // NEW: calculated bounds
    width: u32,
    height: u32,
    colors: Vec<Color>,
}

impl SixelCapture {
    pub fn sequences_in_area(&self, area: Rect) -> Vec<&SixelSequence>;  // NEW
    pub fn sequences_outside_area(&self, area: Rect) -> Vec<&SixelSequence>;  // NEW
    pub fn is_empty(&self) -> bool;  // NEW
    pub fn assert_all_within(&self, area: Rect) -> Result<()>;  // NEW
}
```

### Phase 6 Enhancements: Bevy Integration

**Add to existing Phase 6**:

1. **Bevy-Specific Harness**
   - [ ] Create `BevyTuiTestHarness` wrapper
   - [ ] Support Bevy app lifecycle
   - [ ] Control update cycle execution
   - [ ] Access Bevy World for queries

2. **ECS Testing Utilities**
   - [ ] Query entities by component
   - [ ] Access resources
   - [ ] Send Bevy events
   - [ ] Inspect system execution

3. **bevy_ratatui Helpers**
   - [ ] Integration with bevy_ratatui plugin
   - [ ] Screen state inspection
   - [ ] Event routing verification

**New Module**: `src/bevy.rs`

```rust
#[cfg(feature = "bevy")]
pub mod bevy {
    use bevy::prelude::*;

    pub struct BevyTuiTestHarness {
        harness: TuiTestHarness,
        app: App,
    }

    impl BevyTuiTestHarness {
        pub fn new() -> Result<Self>;
        pub fn with_plugins(plugins: impl Plugins) -> Result<Self>;
        pub fn update(&mut self) -> Result<()>;
        pub fn query<T: Component>(&self) -> Vec<&T>;
        pub fn resource<T: Resource>(&self) -> Option<&T>;
    }
}
```

### Phase 6 Enhancements: High-Level Assertions

**Add assertion helpers**:

```rust
impl RatatuiTestHelper {
    // Text assertions
    pub fn assert_text_at(&self, x: u16, y: u16, text: &str) -> Result<()>;
    pub fn assert_text_contains(&self, text: &str) -> Result<()>;
    pub fn assert_area_contains_text(&self, area: Rect, text: &str) -> Result<()>;

    // Cursor assertions
    pub fn assert_cursor_position(&self, x: u16, y: u16) -> Result<()>;
    pub fn assert_cursor_in_area(&self, area: Rect) -> Result<()>;
    pub fn get_cursor_position(&self) -> (u16, u16);

    // Sixel assertions (delegates to SixelCapture)
    pub fn assert_sixel_at(&self, area: Rect) -> Result<()>;
    pub fn assert_sixel_cleared(&self, area: Rect) -> Result<()>;
    pub fn assert_no_sixel_outside(&self, area: Rect) -> Result<()>;
    pub fn has_sixel_graphics(&self) -> bool;
}
```

### Phase 7 Additions: Debug Support

**Add basic debugging aids**:

- [ ] Save terminal state on test failure
- [ ] Export as ANSI text file
- [ ] Verbose escape sequence logging (opt-in)
- [ ] Diff visualization for assertions

## Feature Flags Update

```toml
[features]
default = []
snapshot-expect = ["expect-test"]
snapshot-insta = ["insta"]
async-tokio = ["tokio"]
async-async-std = ["async-std"]
ratatui-helpers = ["ratatui", "crossterm"]
sixel-image = ["image"]
bevy = ["bevy", "bevy_ecs"]  # NEW
bevy-ratatui = ["bevy", "bevy_ratatui"]  # NEW
full = [
    "snapshot-expect",
    "snapshot-insta",
    "async-tokio",
    "ratatui-helpers",
    "sixel-image",
    "bevy",
]
```

## Dependencies Update

```toml
[dependencies]
# Existing...
portable-pty = "0.8"
vt100 = "0.15"
thiserror = "1.0"

# Optional - existing
insta = { version = "1.34", optional = true }
expect-test = { version = "1.4", optional = true }
tokio = { version = "1.35", optional = true }
async-std = { version = "1.12", optional = true }
ratatui = { version = "0.29", optional = true }  # UPDATE: Specify 0.29.x
crossterm = { version = "0.27", optional = true }
image = { version = "0.24", optional = true }
serde = { version = "1.0", features = ["derive"], optional = true }
serde_json = { version = "1.0", optional = true }

# NEW: Bevy support
bevy = { version = "0.14", optional = true, default-features = false, features = ["bevy_core"] }
bevy_ecs = { version = "0.14", optional = true }
bevy_ratatui = { version = "0.7", optional = true }
```

## Testing Strategy for dgx-pixels

### Recommended Test Structure

```
dgx-pixels/
├── tests/
│   ├── integration/
│   │   ├── sixel_positioning.rs      # Use Case 1
│   │   ├── screen_transitions.rs     # Use Case 2
│   │   ├── text_input.rs             # Use Case 3
│   │   ├── job_management.rs         # Use Case 4
│   │   ├── gallery_screen.rs
│   │   ├── generation_screen.rs
│   │   ├── comparison_screen.rs
│   │   └── ... (all 8 screens)
│   └── fixtures/
│       ├── test-sprite.png
│       └── expected-sixel/
│           └── gallery-preview.six
```

### Example Integration Test

```rust
// tests/integration/sixel_positioning.rs

use term_test::BevyTuiTestHarness;
use dgx_pixels::{App, Screen};

#[tokio::test]
async fn test_sixel_renders_in_preview_area() -> Result<()> {
    // Create Bevy TUI test harness
    let mut test = BevyTuiTestHarness::new()?;

    // Setup: Load test app with image
    test.load_test_image("tests/fixtures/test-sprite.png")?;

    // Navigate to Gallery screen
    test.press_key('2')?;  // Gallery is screen 2
    test.update()?;  // Process navigation
    test.render_frame()?;

    // Get preview area from layout
    let preview_area = test.query_component::<PreviewPanel>()?.area;

    // Assert: Sixel is within bounds
    test.assert_sixel_within(preview_area)?;
    test.assert_no_sixel_outside(preview_area)?;

    // Assert: On correct screen
    test.assert_on_screen(Screen::Gallery)?;

    Ok(())
}

#[tokio::test]
async fn test_sixel_clears_on_navigation() -> Result<()> {
    let mut test = BevyTuiTestHarness::new()?;

    // Setup: Gallery with image
    test.load_test_image("tests/fixtures/test-sprite.png")?;
    test.navigate_to(Screen::Gallery)?;
    test.render_frame()?;

    // Verify image rendered
    assert!(test.has_sixel_graphics());
    let before = test.capture_sixel_state()?;

    // Navigate away
    test.press_key('1')?;  // Generation screen
    test.update()?;
    test.render_frame()?;

    // Assert: Sixel cleared
    assert!(!test.has_sixel_graphics());
    test.assert_on_screen(Screen::Generation)?;

    // Verify state changed
    let after = test.capture_sixel_state()?;
    assert!(after.differs_from(&before));

    Ok(())
}
```

## CI/CD Configuration

### GitHub Actions Workflow

```yaml
name: TUI Integration Tests

on: [push, pull_request]

jobs:
  test-tui:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      # No X11/Wayland needed - term-test is headless!

      - name: Run unit tests
        run: cargo test --lib

      - name: Run integration tests
        run: cargo test --test '*' --features bevy,bevy-ratatui

      - name: Upload failed test screenshots
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: test-failures
          path: target/test-output/*.txt
```

## Success Criteria (Updated)

The term-test library will successfully support dgx-pixels if:

1. ✅ Can detect Sixel positioning bugs (bounds checking)
2. ✅ Can detect Sixel persistence bugs (clearing verification)
3. ✅ Runs headless in CI/CD (no GUI required)
4. ✅ Integrates with Bevy + bevy_ratatui
5. ✅ Supports all dgx-pixels screens (8 screens)
6. ✅ Test execution time < 100ms per test average
7. ✅ Clear failure messages with debugging info
8. ✅ Compatible with ratatui 0.29.x
9. ✅ Supports Tokio async runtime

## Implementation Priority (Revised)

### For dgx-pixels Support

1. **Phase 1**: Core PTY Harness (unchanged)
2. **Phase 2**: Event Simulation (unchanged)
3. **Phase 3**: Snapshot Integration (unchanged)
4. **Phase 4**: Async Support with Tokio (**required for dgx-pixels**)
5. **Phase 5**: Sixel Support **with position tracking** (**critical**)
6. **Phase 6**: Ratatui + **Bevy Integration** (**critical**)
7. **Phase 7**: Polish + Debug Support

### Phase 5 & 6 are Now Critical

The original roadmap marked Phase 5 as P1 (High) and Phase 6 as P2 (Medium).

**For dgx-pixels support**:
- Phase 5: **P0 (Critical)** - Sixel testing is the original motivation
- Phase 6: **P0 (Critical)** - Bevy integration is required

## Next Steps

1. **Validate vt100 capabilities**
   - Does vt100 track cursor position when Sixel is rendered?
   - Can we extract position metadata from parsed sequences?
   - If not, can we extend vt100 or use termwiz?

2. **Prototype Bevy integration**
   - Can we create headless Bevy app for testing?
   - How to control Bevy update cycle frame-by-frame?
   - How to access World/ECS from test harness?

3. **Design position tracking**
   - How to map Sixel escape sequences to terminal coordinates?
   - How to track graphics layer separately from text?
   - How to detect clearing sequences?

4. **Update roadmap**
   - Incorporate dgx-pixels requirements
   - Adjust phase priorities
   - Add new tasks to phases

## References

- **Issue #1**: TUI Integration Testing Framework Requirements
- **dgx-pixels**: https://github.com/raibid-labs/dgx-pixels
- **bevy_ratatui**: https://github.com/cxreiff/bevy_ratatui
- **Bevy ECS**: https://bevyengine.org/learn/book/getting-started/ecs/

---

**Document Status**: Analysis Complete
**Next Action**: Update ROADMAP.md with Bevy integration and Sixel enhancements
**Priority**: Phase 5 & 6 elevated to P0 (Critical)

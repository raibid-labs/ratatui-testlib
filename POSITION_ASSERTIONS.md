# Position and Layout Assertions - Implementation Summary

## Issue #11: Add assertions for UI component positioning and layout

### Overview

Implemented comprehensive position and layout assertion APIs for testing TUI components with complex layouts (tab bars, overlays, panels). This enables developers to verify that components render in the correct positions and don't overlap incorrectly.

### API Design

#### New Types

**`Rect`** - A rectangular area in terminal coordinate space:
```rust
pub struct Rect {
    pub x: u16,      // Column (0-indexed)
    pub y: u16,      // Row (0-indexed)
    pub width: u16,  // Width in columns
    pub height: u16, // Height in rows
}
```

Compatible with `ratatui::layout::Rect` for seamless integration.

**Methods:**
- `new(x, y, width, height)` - Create a rectangle
- `right()` - Get right edge (x + width)
- `bottom()` - Get bottom edge (y + height)
- `contains(x, y)` - Check if point is inside
- `contains_rect(other)` - Check if rectangle is fully contained
- `intersects(other)` - Check if rectangles overlap

**`Axis`** - Alignment axis enum:
```rust
pub enum Axis {
    Horizontal,  // Same Y coordinate
    Vertical,    // Same X coordinate
}
```

#### Assertion Methods

**1. `assert_text_at_position(text, row, col)`**

Verifies text appears at an exact position.

```rust
harness.assert_text_at_position("Tab 1", 22, 0)?;
```

**Error Message:**
```
Text mismatch at position (22, 0)
  Expected: "Tab 1"
  Found:    "Tab 2"

Screen state:
[full screen dump]
```

**2. `assert_text_within_bounds(text, area)`**

Searches for text anywhere within a rectangular area.

```rust
let preview_area = Rect::new(5, 40, 35, 15);
harness.assert_text_within_bounds("Preview", preview_area)?;
```

**Error Message:**
```
Text "Preview" not found within bounds (x=5, y=40, width=35, height=15)

Screen state:
[full screen dump]
```

**3. `assert_no_overlap(rect1, rect2)`**

Verifies two rectangles don't intersect.

```rust
let sidebar = Rect::new(0, 0, 20, 24);
let preview = Rect::new(20, 0, 60, 24);
harness.assert_no_overlap(sidebar, preview)?;
```

**Error Message:**
```
Rectangles overlap!
Rect 1: (x=0, y=0, width=20, height=24)
Rect 2: (x=15, y=0, width=60, height=24)
Overlap region exists between x=[15, 20) and y=[0, 24)
```

**4. `assert_aligned(rect1, rect2, axis)`**

Verifies rectangles are aligned on an axis.

```rust
let button1 = Rect::new(10, 20, 15, 3);
let button2 = Rect::new(30, 20, 15, 3);
harness.assert_aligned(button1, button2, Axis::Horizontal)?;
```

**Error Message:**
```
Rectangles not horizontally aligned (different Y coordinates)
Rect 1: y=20 (x=10, width=15, height=3)
Rect 2: y=21 (x=30, width=15, height=3)
```

### Use Cases

#### Tab Bar at Bottom

```rust
#[test]
fn test_tab_bar_at_bottom() -> Result<()> {
    let harness = TuiTestHarness::new(80, 24)?;
    // ... render UI ...

    let tab_bar_area = Rect::new(0, 22, 80, 2);
    harness.assert_text_within_bounds("Tab 1", tab_bar_area)?;
    harness.assert_text_within_bounds("Tab 2", tab_bar_area)?;
    Ok(())
}
```

#### Overlay Within Preview Area

```rust
#[test]
fn test_overlay_within_preview_area() -> Result<()> {
    let harness = TuiTestHarness::new(80, 24)?;
    let preview = Rect::new(10, 5, 50, 20);

    // Assert text appears within bounds
    harness.assert_text_within_bounds("Preview", preview)?;

    // Assert Sixel graphics within bounds (if sixel feature enabled)
    #[cfg(feature = "sixel")]
    harness.assert_sixel_within_bounds((preview.y, preview.x, preview.width, preview.height))?;

    Ok(())
}
```

#### Complex Layout Verification

```rust
#[test]
fn test_complex_layout() -> Result<()> {
    let harness = TuiTestHarness::new(80, 24)?;

    // Define layout areas
    let header = Rect::new(0, 0, 80, 2);
    let sidebar = Rect::new(0, 2, 20, 20);
    let content = Rect::new(20, 2, 60, 20);
    let status_bar = Rect::new(0, 22, 80, 2);

    // Verify no overlap
    harness.assert_no_overlap(sidebar, content)?;
    harness.assert_no_overlap(header, status_bar)?;

    // Verify alignment
    harness.assert_aligned(sidebar, content, Axis::Horizontal)?;

    Ok(())
}
```

### Files Modified

1. **`src/screen.rs`** - Added `Rect` type with geometric operations
2. **`src/harness.rs`** - Added assertion methods and `Axis` enum
3. **`src/lib.rs`** - Exported new types (`Rect`, `Axis`)
4. **`examples/position_test.rs`** - Created comprehensive example

### Test Coverage

Added 18 comprehensive tests:

**Rect Operations:**
- `test_rect_creation` - Basic construction
- `test_rect_edges` - Edge calculations
- `test_rect_contains_point` - Point containment
- `test_rect_contains_rect` - Rectangle containment
- `test_rect_intersects` - Intersection detection

**Position Assertions:**
- `test_assert_text_at_position_success` - Exact position matching
- `test_assert_text_at_position_failure` - Error handling
- `test_assert_text_at_position_out_of_bounds` - Bounds checking
- `test_assert_text_within_bounds_success` - Area search
- `test_assert_text_within_bounds_failure` - Not found handling
- `test_assert_text_within_bounds_tab_bar` - Tab bar use case

**Layout Assertions:**
- `test_assert_no_overlap_success` - Non-overlapping rects
- `test_assert_no_overlap_failure` - Overlap detection
- `test_assert_aligned_horizontal_success` - Horizontal alignment
- `test_assert_aligned_horizontal_failure` - Misalignment detection
- `test_assert_aligned_vertical_success` - Vertical alignment
- `test_assert_aligned_vertical_failure` - Misalignment detection

**Complex Scenarios:**
- `test_complex_layout_assertions` - Multi-area layout
- `test_multiline_text_in_area` - Multiline content
- `test_edge_case_text_at_screen_edge` - Screen boundaries
- `test_text_spanning_multiple_lines` - Line-by-line text

### Test Results

```
running 102 tests
...
test result: ok. 102 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

All tests pass, including the 18 new position assertion tests.

### Error Messages

All assertion methods provide **clear, actionable error messages** that include:

1. **What went wrong** - Clear description of the failure
2. **Expected vs Actual** - What was expected and what was found
3. **Position/Coordinates** - Exact locations for debugging
4. **Full screen state** - Complete terminal dump for context

Example:
```
Text mismatch at position (4, 9)
  Expected: "World"
  Found:    "Hello"

Screen state:




         Hello World
[... rest of screen ...]
```

### Coordination with Other Issues

- **Built on Wave 2's screen state API** - Uses `get_cell()` for inspection
- **Compatible with Sixel bounds** - Can be used alongside Sixel assertions
- **Follows existing patterns** - Uses same `Result<()>` and error types
- **No dependencies on Agent C1** - Standalone implementation

### Example Run

```bash
$ cargo run --example position_test

Position and Layout Assertion Example

1. Simulating complex TUI layout...
   Layout created

2. Testing text at specific positions...
   ✓ Found 'Files' at (2, 0)
   ✓ Found 'Content Area' at (2, 24)
   ✓ Found 'Status: Ready' at (23, 0)

3. Testing text within bounds...
   ✓ Found 'file1.txt' in sidebar area
   ✓ Found 'main content' in content area
   ✓ Found all tabs in tab bar area

4. Testing no overlap between areas...
   ✓ Sidebar and content don't overlap
   ✓ Header and status bar don't overlap

5. Testing alignment...
   ✓ Sidebar and content are horizontally aligned
   ✓ Buttons are horizontally aligned

✅ All position assertions passed!
```

### Acceptance Criteria Status

- ✅ `assert_within_bounds(text, rect)` - Text appears within rectangle
- ✅ `assert_at_position(text, x, y)` - Text at exact position
- ✅ `assert_no_overlap(rect1, rect2)` - Rectangles don't overlap
- ✅ `assert_aligned(rect1, rect2, axis)` - Rectangles aligned
- ✅ Support for Rect type (compatible with ratatui::layout::Rect)
- ✅ Clear error messages showing expected vs actual positions
- ✅ Tests covering various layout scenarios

### Documentation

- Comprehensive doc comments on all public items
- Examples in doc comments
- Full example in `examples/position_test.rs`
- Error message examples in comments

### Priority

**MEDIUM** priority - Very useful for complex UIs, particularly for:
- Tab bars and navigation
- Multi-pane layouts
- Overlay positioning
- Component alignment verification

### Future Enhancements

Potential additions for post-MVP:
- `assert_centered(rect, container)` - Verify centering
- `assert_spacing(rect1, rect2, spacing)` - Verify spacing between components
- `assert_within_margin(rect1, rect2, margin)` - Verify margins
- Regex-based text search within bounds
- Multi-rect alignment checks (3+ rectangles)

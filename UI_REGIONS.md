# UI Region Testing Helpers

This document describes the UI region testing helpers added in Issue #51.

## Overview

The `regions` module provides abstractions for testing fixed UI regions (status bars, tab bars, sidebars, chrome) that overlay or partition terminal content in TUI applications.

## Key Types

### `UiRegion`

Defines a fixed UI region with:
- `name`: Region identifier (e.g., "status_bar", "tab_bar")
- `anchor`: Where the region is positioned (`Top`, `Bottom`, `Left`, `Right`)
- `size`: Height (for Top/Bottom) or width (for Left/Right) in cells

### `RegionAnchor`

Enum for region positioning:
- `Top`: Anchored to the top of the screen
- `Bottom`: Anchored to the bottom of the screen
- `Left`: Anchored to the left of the screen
- `Right`: Anchored to the right of the screen

### `RegionBounds`

Represents rectangular region bounds with:
- `row`: Starting row (0-indexed)
- `col`: Starting column (0-indexed)
- `width`: Width in columns
- `height`: Height in rows

Methods:
- `contains(row, col)`: Check if a position is within bounds
- `intersects(other)`: Check if two regions overlap

### `UiRegionTester`

Builder for defining and querying multiple regions:
- `new(width, height)`: Create tester with screen dimensions
- `with_status_bar(height)`: Add bottom-anchored status bar
- `with_tab_bar(height)`: Add top-anchored tab bar
- `with_left_sidebar(width)`: Add left-anchored sidebar
- `with_right_sidebar(width)`: Add right-anchored sidebar
- `with_region(region)`: Add custom region
- `region_bounds(name)`: Get bounds for named region
- `content_area()`: Calculate remaining space after fixed regions
- `is_in_region(name, row, col)`: Check if position is in region
- `is_in_content_area(row, col)`: Check if position is in content area

### `UiRegionTestExt` (requires `scarab` feature)

Extension trait for `ScarabTestHarness`:
- `region_contents(tester, region_name)`: Extract region grid contents
- `content_area_contents(tester)`: Get content excluding fixed regions
- `assert_not_in_region(tester, region_name, text)`: Verify text not in region
- `assert_region_contains(tester, region_name, text)`: Verify text in region
- `verify_resize(tester, width, height)`: Test resize with region recalculation

## Usage Example

```rust
use ratatui_testlib::regions::{UiRegionTester, UiRegionTestExt};
use ratatui_testlib::scarab::ScarabTestHarness;

// Define UI regions
let tester = UiRegionTester::new(80, 24)
    .with_status_bar(1)
    .with_tab_bar(2)
    .with_left_sidebar(20);

// Calculate content area
let content = tester.content_area();
assert_eq!(content.row, 2);     // Below tab bar
assert_eq!(content.col, 20);    // Right of sidebar
assert_eq!(content.height, 21); // 24 - 2 (tab) - 1 (status)
assert_eq!(content.width, 60);  // 80 - 20 (sidebar)

// Check positions
assert!(tester.is_in_region("status_bar", 23, 0));
assert!(tester.is_in_content_area(10, 40));

// With Scarab harness
let mut harness = ScarabTestHarness::connect()?;

// Get status bar contents
let status = harness.region_contents(&tester, "status_bar")?;
assert!(status.contains("Ready"));

// Verify text not in status bar
harness.assert_not_in_region(&tester, "status_bar", "ERROR")?;

// Test resize
harness.verify_resize(&mut tester, 100, 30)?;
```

## Testing Patterns

### Pattern 1: Fixed Chrome Testing

Verify that UI chrome (status bars, tabs) doesn't interfere with content:

```rust
let tester = UiRegionTester::new(80, 24)
    .with_status_bar(1)
    .with_tab_bar(2);

// Verify error messages don't leak into chrome
harness.assert_not_in_region(&tester, "status_bar", "FATAL")?;
harness.assert_not_in_region(&tester, "tab_bar", "ERROR")?;
```

### Pattern 2: Layout Verification

Test that content appears in the correct region:

```rust
let tester = UiRegionTester::new(80, 24)
    .with_left_sidebar(20);

// Verify file tree is in sidebar
let sidebar_content = harness.region_contents(&tester, "left_sidebar")?;
assert!(sidebar_content.contains("src/"));
assert!(sidebar_content.contains("tests/"));

// Verify main content is in content area
let content = harness.content_area_contents(&tester)?;
assert!(content.contains("fn main()"));
```

### Pattern 3: Resize Testing

Verify regions recalculate correctly on resize:

```rust
let mut tester = UiRegionTester::new(80, 24)
    .with_status_bar(1);

// Initial content area
let initial = tester.content_area();
assert_eq!(initial.height, 23);

// Resize and verify
harness.verify_resize(&mut tester, 100, 30)?;
let resized = tester.content_area();
assert_eq!(resized.width, 100);
assert_eq!(resized.height, 29); // 30 - 1 (status)
```

### Pattern 4: Custom Regions

Define application-specific regions:

```rust
use ratatui_testlib::regions::{UiRegion, RegionAnchor};

let notification = UiRegion {
    name: "notification".to_string(),
    anchor: RegionAnchor::Top,
    size: 3,
};

let tester = UiRegionTester::new(80, 24)
    .with_region(notification)
    .with_status_bar(1);

// Test notification area
harness.assert_region_contains(&tester, "notification", "3 new messages")?;
```

## Implementation Details

### Region Calculation Order

Regions are applied in order: Top → Bottom → Left → Right

This means:
1. Top regions (tabs, headers) span full width
2. Bottom regions (status bars) span full width
3. Left regions start below top regions, end above bottom regions
4. Right regions start below top regions, end above bottom regions
5. Content area is what remains

### Coordinate System

- Rows and columns are 0-indexed
- Row 0 is the top of the screen
- Column 0 is the left of the screen
- Regions use inclusive start, exclusive end

### Edge Cases

- Regions larger than screen: Size is capped to available space
- Overlapping regions: Later regions are positioned after earlier ones
- Zero-size regions: Valid but don't occupy space
- Empty content area: When regions consume entire screen

## Examples

See `examples/ui_regions_test.rs` for comprehensive demonstrations of:
- Basic region setup
- Position checking
- Custom regions
- Region intersection testing
- Scarab integration (when available)

## Tests

Unit tests: `src/regions.rs` (18 tests)
Integration tests: `tests/regions_integration_test.rs` (12 tests)

All tests pass with 100% coverage of the public API.

## Feature Flags

- `ipc`: Required for `UiRegion`, `UiRegionTester`, `RegionBounds`
- `scarab`: Required for `UiRegionTestExt` trait

## Documentation

Full API documentation available via:
```bash
cargo doc --features scarab --open
```

Navigate to `ratatui_testlib::regions`.

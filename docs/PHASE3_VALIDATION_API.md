# Phase 3 Validation API Specification

**Version**: 1.0
**Date**: 2025-11-21
**Status**: Design Document

## Overview

This document specifies the validation APIs for Sixel graphics testing in terminal-testlib. These APIs enable developers to verify that Sixel graphics appear in the correct locations and stay within designated bounds.

---

## API Design Principles

1. **Ergonomic**: Simple, intuitive method names
2. **Composable**: Methods work well together
3. **Informative**: Clear error messages with context
4. **Flexible**: Support various testing patterns
5. **Type-safe**: Leverage Rust's type system

---

## Core Types

### SixelRegion

**Location**: `src/screen.rs`

```rust
/// Represents a Sixel graphics region in the terminal.
///
/// Tracks the position, dimensions, and raw data of a Sixel
/// graphic rendered on the screen.
#[derive(Debug, Clone)]
pub struct SixelRegion {
    /// Starting row (0-indexed).
    pub start_row: u16,

    /// Starting column (0-indexed).
    pub start_col: u16,

    /// Width in pixels.
    pub width: u32,

    /// Height in pixels.
    pub height: u32,

    /// Width in terminal cells (calculated from pixels).
    pub width_cells: u16,

    /// Height in terminal cells (calculated from pixels).
    pub height_cells: u16,

    /// Raw Sixel escape sequence data.
    pub data: Vec<u8>,
}

impl SixelRegion {
    /// Check if this region is completely within the given area (cell-based).
    ///
    /// # Arguments
    ///
    /// * `area` - Area as (row, col, width, height) in cells
    ///
    /// # Returns
    ///
    /// `true` if the entire region fits within the area, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// let region = SixelRegion { /* ... */ };
    /// let preview_area = (5, 30, 70, 30);
    ///
    /// if region.is_within_cells(preview_area) {
    ///     println!("Image is within preview area");
    /// }
    /// ```
    pub fn is_within_cells(&self, area: (u16, u16, u16, u16)) -> bool;

    /// Check if this region overlaps with the given area (cell-based).
    ///
    /// # Arguments
    ///
    /// * `area` - Area as (row, col, width, height) in cells
    ///
    /// # Returns
    ///
    /// `true` if any part of the region intersects with the area.
    ///
    /// # Example
    ///
    /// ```rust
    /// let region = SixelRegion { /* ... */ };
    /// let sidebar_area = (0, 0, 30, 40);
    ///
    /// if region.overlaps_cells(sidebar_area) {
    ///     eprintln!("Warning: Image overlaps sidebar");
    /// }
    /// ```
    pub fn overlaps_cells(&self, area: (u16, u16, u16, u16)) -> bool;

    /// Get the end position of this region (exclusive).
    ///
    /// # Returns
    ///
    /// Tuple of (end_row, end_col) representing the cell just past the region.
    ///
    /// # Example
    ///
    /// ```rust
    /// let region = SixelRegion {
    ///     start_row: 10,
    ///     start_col: 20,
    ///     width_cells: 15,
    ///     height_cells: 10,
    ///     // ...
    /// };
    ///
    /// let (end_row, end_col) = region.end_position();
    /// assert_eq!(end_row, 20);  // 10 + 10
    /// assert_eq!(end_col, 35);  // 20 + 15
    /// ```
    pub fn end_position(&self) -> (u16, u16) {
        (
            self.start_row + self.height_cells,
            self.start_col + self.width_cells,
        )
    }

    /// Get the cell-based bounding box for this region.
    ///
    /// # Returns
    ///
    /// Tuple of (row, col, width, height) in cells.
    pub fn bounds_cells(&self) -> (u16, u16, u16, u16) {
        (
            self.start_row,
            self.start_col,
            self.width_cells,
            self.height_cells,
        )
    }

    /// Check if this region is at the exact position.
    ///
    /// # Arguments
    ///
    /// * `row` - Row to check (0-indexed)
    /// * `col` - Column to check (0-indexed)
    ///
    /// # Returns
    ///
    /// `true` if the region starts at exactly (row, col).
    pub fn is_at(&self, row: u16, col: u16) -> bool {
        self.start_row == row && self.start_col == col
    }
}
```

### SixelSequence

**Location**: `src/sixel.rs`

```rust
/// Represents a captured Sixel sequence with position information.
///
/// This is the type used by SixelCapture for querying and validation.
#[derive(Debug, Clone, PartialEq)]
pub struct SixelSequence {
    /// Raw Sixel escape sequence bytes (including DCS wrapper).
    pub raw: Vec<u8>,

    /// Cursor position when the Sixel was rendered (row, col).
    pub position: (u16, u16),

    /// Calculated bounding rectangle (row, col, width, height) in cells.
    pub bounds: (u16, u16, u16, u16),
}

impl SixelSequence {
    // Existing methods...
    pub fn new(raw: Vec<u8>, position: (u16, u16), bounds: (u16, u16, u16, u16)) -> Self;
    pub fn is_within(&self, area: (u16, u16, u16, u16)) -> bool;
    pub fn overlaps(&self, area: (u16, u16, u16, u16)) -> bool;
}
```

### SixelCapture

**Location**: `src/sixel.rs`

```rust
/// Captures all Sixel sequences from terminal output.
///
/// Provides methods for querying and validating Sixel graphics
/// in the terminal screen state.
#[derive(Debug, Clone, PartialEq)]
pub struct SixelCapture {
    sequences: Vec<SixelSequence>,
}

impl SixelCapture {
    // Existing methods...
    pub fn new() -> Self;
    pub fn from_screen_state(screen: &ScreenState) -> Self;
    pub fn sequences(&self) -> &[SixelSequence];
    pub fn is_empty(&self) -> bool;
    pub fn sequences_in_area(&self, area: (u16, u16, u16, u16)) -> Vec<&SixelSequence>;
    pub fn sequences_outside_area(&self, area: (u16, u16, u16, u16)) -> Vec<&SixelSequence>;
    pub fn assert_all_within(&self, area: (u16, u16, u16, u16)) -> Result<()>;
    pub fn differs_from(&self, other: &SixelCapture) -> bool;

    // New methods for Phase 3...
}
```

---

## New API Methods

### 1. Harness-Level Validation

**Location**: `src/harness.rs` - Add to `TuiTestHarness`

#### assert_sixel_within_bounds()

```rust
impl TuiTestHarness {
    /// Assert that all Sixel graphics are within the specified area.
    ///
    /// This is the primary validation method for ensuring that graphics
    /// appear in their designated regions (e.g., preview panels).
    ///
    /// # Arguments
    ///
    /// * `area` - Bounding area as (row, col, width, height) in cells
    ///
    /// # Errors
    ///
    /// Returns `TermTestError::SixelValidation` if any Sixel graphic
    /// extends outside the specified area. The error message includes:
    /// - Number of graphics outside bounds
    /// - Positions of out-of-bounds graphics
    /// - Expected area definition
    ///
    /// # Example
    ///
    /// ```rust
    /// use term_test::{TuiTestHarness, Result};
    ///
    /// fn test_preview_area() -> Result<()> {
    ///     let mut harness = TuiTestHarness::new(120, 40)?;
    ///     // ... render graphics ...
    ///
    ///     // Define preview panel: rows 5-34, cols 30-99
    ///     let preview_area = (5, 30, 70, 30);
    ///
    ///     // Verify all graphics within preview
    ///     harness.assert_sixel_within_bounds(preview_area)?;
    ///
    ///     Ok(())
    /// }
    /// ```
    pub fn assert_sixel_within_bounds(&self, area: (u16, u16, u16, u16)) -> Result<()> {
        let capture = SixelCapture::from_screen_state(&self.state);
        capture.assert_all_within(area)
    }
}
```

#### get_sixel_at()

```rust
impl TuiTestHarness {
    /// Get the Sixel region at the specified position.
    ///
    /// Returns the first Sixel region whose starting position matches
    /// the given coordinates. Useful for verifying specific graphics.
    ///
    /// # Arguments
    ///
    /// * `row` - Row to query (0-indexed)
    /// * `col` - Column to query (0-indexed)
    ///
    /// # Returns
    ///
    /// `Some(&SixelRegion)` if a region starts at the position, `None` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use term_test::TuiTestHarness;
    /// # fn test() -> term_test::Result<()> {
    /// let harness = TuiTestHarness::new(80, 24)?;
    /// // ... render Sixel at (10, 20) ...
    ///
    /// if let Some(region) = harness.get_sixel_at(10, 20) {
    ///     println!("Found Sixel: {}x{} pixels",
    ///         region.width, region.height);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn get_sixel_at(&self, row: u16, col: u16) -> Option<&SixelRegion> {
        self.state.sixel_regions()
            .iter()
            .find(|r| r.start_row == row && r.start_col == col)
    }
}
```

#### sixel_count()

```rust
impl TuiTestHarness {
    /// Returns the number of Sixel graphics currently on screen.
    ///
    /// Convenience method for checking graphic presence.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use term_test::TuiTestHarness;
    /// # fn test() -> term_test::Result<()> {
    /// let mut harness = TuiTestHarness::new(80, 24)?;
    ///
    /// // Wait for image to appear
    /// harness.wait_for(|_| harness.sixel_count() > 0)?;
    ///
    /// assert_eq!(harness.sixel_count(), 1);
    /// # Ok(())
    /// # }
    /// ```
    pub fn sixel_count(&self) -> usize {
        self.state.sixel_regions().len()
    }
}
```

#### verify_sixel_cleared()

```rust
impl TuiTestHarness {
    /// Verify that Sixel graphics have been cleared since a previous snapshot.
    ///
    /// Useful for testing screen transitions to ensure graphics are properly
    /// cleared when navigating away from image views.
    ///
    /// # Arguments
    ///
    /// * `previous` - Previous `SixelCapture` snapshot to compare against
    ///
    /// # Returns
    ///
    /// `true` if the Sixel state has changed (cleared or modified),
    /// `false` if identical.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use term_test::{TuiTestHarness, SixelCapture};
    /// # fn test() -> term_test::Result<()> {
    /// let mut harness = TuiTestHarness::new(80, 24)?;
    /// // ... render image in Gallery screen ...
    ///
    /// let before = SixelCapture::from_screen_state(&harness.state());
    ///
    /// // Navigate away
    /// harness.send_key(KeyCode::Esc)?;
    /// harness.wait_for_text("Main Menu")?;
    ///
    /// let after = SixelCapture::from_screen_state(&harness.state());
    ///
    /// assert!(after.differs_from(&before), "Graphics should be cleared");
    /// # Ok(())
    /// # }
    /// ```
    pub fn verify_sixel_cleared(&self, previous: &SixelCapture) -> bool {
        let current = SixelCapture::from_screen_state(&self.state);
        current.differs_from(previous)
    }
}
```

### 2. Enhanced SixelCapture Methods

**Location**: `src/sixel.rs` - Add to `SixelCapture`

#### sequences_overlapping()

```rust
impl SixelCapture {
    /// Get all sequences that overlap with the specified area.
    ///
    /// Unlike `sequences_in_area()` which requires complete containment,
    /// this method finds any sequences with partial or full overlap.
    ///
    /// # Arguments
    ///
    /// * `area` - Area as (row, col, width, height)
    ///
    /// # Returns
    ///
    /// Vector of references to overlapping sequences.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use term_test::SixelCapture;
    /// let capture = SixelCapture::new();
    /// let sidebar = (0, 0, 30, 40);
    ///
    /// let overlapping = capture.sequences_overlapping(sidebar);
    /// if !overlapping.is_empty() {
    ///     eprintln!("Warning: {} graphics overlap sidebar", overlapping.len());
    /// }
    /// ```
    pub fn sequences_overlapping(&self, area: (u16, u16, u16, u16)) -> Vec<&SixelSequence> {
        self.sequences
            .iter()
            .filter(|seq| seq.overlaps(area))
            .collect()
    }
}
```

#### sequences_at_row()

```rust
impl SixelCapture {
    /// Get all sequences that start at the specified row.
    ///
    /// Useful for finding graphics in a specific vertical region.
    ///
    /// # Arguments
    ///
    /// * `row` - Row number (0-indexed)
    ///
    /// # Returns
    ///
    /// Vector of references to sequences starting at the row.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use term_test::SixelCapture;
    /// let capture = SixelCapture::new();
    /// let sequences = capture.sequences_at_row(10);
    ///
    /// for seq in sequences {
    ///     println!("Sixel at row 10, col {}", seq.position.1);
    /// }
    /// ```
    pub fn sequences_at_row(&self, row: u16) -> Vec<&SixelSequence> {
        self.sequences
            .iter()
            .filter(|seq| seq.position.0 == row)
            .collect()
    }
}
```

#### has_sequences_in()

```rust
impl SixelCapture {
    /// Check if any sequences exist in the given area.
    ///
    /// Convenience method that returns a boolean instead of a vector.
    ///
    /// # Arguments
    ///
    /// * `area` - Area as (row, col, width, height)
    ///
    /// # Returns
    ///
    /// `true` if at least one sequence is completely within the area.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use term_test::SixelCapture;
    /// let capture = SixelCapture::new();
    /// let preview = (5, 30, 70, 30);
    ///
    /// if capture.has_sequences_in(preview) {
    ///     println!("Graphics present in preview area");
    /// }
    /// ```
    pub fn has_sequences_in(&self, area: (u16, u16, u16, u16)) -> bool {
        !self.sequences_in_area(area).is_empty()
    }
}
```

#### total_coverage()

```rust
impl SixelCapture {
    /// Calculate the total area covered by all Sixel graphics.
    ///
    /// Sums the cell area of all sequences. Note that this counts
    /// overlapping areas multiple times.
    ///
    /// # Returns
    ///
    /// Total coverage in cells (width × height summed).
    ///
    /// # Example
    ///
    /// ```rust
    /// # use term_test::SixelCapture;
    /// let capture = SixelCapture::new();
    /// let coverage = capture.total_coverage();
    ///
    /// println!("Total graphic area: {} cells", coverage);
    /// ```
    pub fn total_coverage(&self) -> u32 {
        self.sequences
            .iter()
            .map(|seq| {
                let (_, _, w, h) = seq.bounds;
                w as u32 * h as u32
            })
            .sum()
    }
}
```

#### bounding_box()

```rust
impl SixelCapture {
    /// Get the bounding box that contains all Sixel graphics.
    ///
    /// Computes the minimum rectangle that encompasses all sequences.
    ///
    /// # Returns
    ///
    /// `Some((row, col, width, height))` if any sequences exist,
    /// `None` if the capture is empty.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use term_test::SixelCapture;
    /// let capture = SixelCapture::new();
    ///
    /// if let Some((r, c, w, h)) = capture.bounding_box() {
    ///     println!("All graphics fit in {}x{} area at ({}, {})",
    ///         w, h, r, c);
    /// }
    /// ```
    pub fn bounding_box(&self) -> Option<(u16, u16, u16, u16)> {
        if self.sequences.is_empty() {
            return None;
        }

        let mut min_row = u16::MAX;
        let mut min_col = u16::MAX;
        let mut max_row = 0u16;
        let mut max_col = 0u16;

        for seq in &self.sequences {
            let (r, c, w, h) = seq.bounds;
            min_row = min_row.min(r);
            min_col = min_col.min(c);
            max_row = max_row.max(r + h);
            max_col = max_col.max(c + w);
        }

        Some((min_row, min_col, max_col - min_col, max_row - min_row))
    }
}
```

---

## Usage Patterns

### Pattern 1: Basic Bounds Validation

```rust
#[test]
fn test_image_preview_bounds() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;
    // ... spawn app, navigate to Gallery ...

    // Define preview area
    let preview_area = (5, 30, 70, 30);

    // Wait for image
    harness.wait_for(|_| harness.sixel_count() > 0)?;

    // Validate bounds
    harness.assert_sixel_within_bounds(preview_area)?;

    Ok(())
}
```

### Pattern 2: Multiple Region Validation

```rust
#[test]
fn test_sidebar_and_preview() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    let sidebar = (0, 0, 30, 40);
    let preview = (5, 30, 70, 30);

    let capture = SixelCapture::from_screen_state(&harness.state());

    // Verify no graphics in sidebar
    assert!(capture.sequences_in_area(sidebar).is_empty(),
        "Sidebar should not contain graphics");

    // Verify graphics in preview
    assert!(!capture.sequences_in_area(preview).is_empty(),
        "Preview should contain graphics");

    Ok(())
}
```

### Pattern 3: Screen Transition Testing

```rust
#[test]
fn test_gallery_to_main_transition() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    // Gallery screen with image
    harness.wait_for(|_| harness.sixel_count() > 0)?;
    let gallery_capture = SixelCapture::from_screen_state(&harness.state());

    // Navigate away
    harness.send_key(KeyCode::Esc)?;
    harness.wait_for_text("Main Menu")?;

    // Verify graphics cleared
    assert!(harness.verify_sixel_cleared(&gallery_capture),
        "Graphics should be cleared on transition");

    Ok(())
}
```

### Pattern 4: Specific Position Verification

```rust
#[test]
fn test_thumbnail_grid() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    // Expected thumbnail positions
    let positions = [
        (10, 35),
        (10, 55),
        (10, 75),
    ];

    // Verify each thumbnail exists
    for (row, col) in positions {
        let region = harness.get_sixel_at(row, col)
            .ok_or_else(|| format!("No Sixel at ({}, {})", row, col))?;

        println!("Thumbnail at ({}, {}): {}x{} pixels",
            row, col, region.width, region.height);
    }

    Ok(())
}
```

### Pattern 5: Coverage Analysis

```rust
#[test]
fn test_graphics_coverage() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    let capture = SixelCapture::from_screen_state(&harness.state());

    // Check coverage statistics
    let total_coverage = capture.total_coverage();
    let sequence_count = capture.sequences().len();
    let avg_size = total_coverage / sequence_count as u32;

    println!("Graphics statistics:");
    println!("  Count: {}", sequence_count);
    println!("  Total coverage: {} cells", total_coverage);
    println!("  Average size: {} cells", avg_size);

    // Verify reasonable coverage
    assert!(total_coverage < 4800, "Coverage too large (>120x40)");

    Ok(())
}
```

---

## Error Types

### SixelValidation Error

**Location**: `src/error.rs`

```rust
#[derive(Debug, thiserror::Error)]
pub enum TermTestError {
    // ... existing variants ...

    /// Sixel validation failed
    #[error("Sixel validation failed: {0}")]
    SixelValidation(String),
}
```

### Error Message Format

**Example 1: Out of Bounds**
```
Error: SixelValidation
  Found 2 Sixel sequence(s) outside area (5, 30, 70, 30):
  Positions: (2, 10), (40, 95)
  Hint: Check cursor positioning before rendering
```

**Example 2: Missing Graphics**
```
Error: Timeout
  Condition not met within 5000ms: wait for Sixel graphics
  Current state: 0 Sixel sequences on screen
  Hint: Verify Sixel rendering is triggered
```

**Example 3: Overlapping Graphics**
```
Warning: 3 Sixel sequence(s) overlap with sidebar area (0, 0, 30, 40)
  Positions: (5, 25), (15, 28), (20, 20)
```

---

## Wait Condition Helpers

### Location: `src/wait.rs` (new module)

```rust
/// Helper functions for common wait conditions involving Sixel graphics.
pub mod sixel {
    use crate::{ScreenState, SixelCapture};

    /// Wait condition: at least one Sixel graphic is present.
    pub fn any_present() -> impl Fn(&ScreenState) -> bool {
        |state| !state.sixel_regions().is_empty()
    }

    /// Wait condition: specific number of Sixel graphics.
    pub fn count_equals(n: usize) -> impl Fn(&ScreenState) -> bool {
        move |state| state.sixel_regions().len() == n
    }

    /// Wait condition: Sixel graphics within specific area.
    pub fn within_area(area: (u16, u16, u16, u16)) -> impl Fn(&ScreenState) -> bool {
        move |state| {
            let capture = SixelCapture::from_screen_state(state);
            capture.sequences_outside_area(area).is_empty()
        }
    }

    /// Wait condition: no Sixel graphics present (cleared).
    pub fn cleared() -> impl Fn(&ScreenState) -> bool {
        |state| state.sixel_regions().is_empty()
    }
}
```

### Usage

```rust
use term_test::wait::sixel;

// Wait for any graphic to appear
harness.wait_for(sixel::any_present())?;

// Wait for specific count
harness.wait_for(sixel::count_equals(3))?;

// Wait for graphics in area
let preview = (5, 30, 70, 30);
harness.wait_for(sixel::within_area(preview))?;

// Wait for graphics to be cleared
harness.wait_for(sixel::cleared())?;
```

---

## Testing the Validation APIs

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_is_within_cells() {
        let region = SixelRegion {
            start_row: 10,
            start_col: 20,
            width: 100,
            height: 60,
            width_cells: 13,  // ceil(100/8)
            height_cells: 10, // ceil(60/6)
            data: vec![],
        };

        // Completely within
        assert!(region.is_within_cells((0, 0, 50, 30)));

        // Partially outside
        assert!(!region.is_within_cells((0, 0, 30, 15)));

        // Completely outside
        assert!(!region.is_within_cells((50, 50, 10, 10)));
    }

    #[test]
    fn test_sixel_capture_bounding_box() {
        let mut capture = SixelCapture::new();
        // Add sequences...

        let bbox = capture.bounding_box().unwrap();
        assert_eq!(bbox, (5, 10, 30, 20));
    }
}
```

### Integration Tests

```rust
#[test]
fn test_harness_validation_apis() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;

    // Render Sixel
    harness.send_text("\x1b[10;10H")?;
    harness.send_text("\x1bPq\"1;1;100;50#0~\x1b\\")?;
    harness.update_state()?;

    // Test count
    assert_eq!(harness.sixel_count(), 1);

    // Test get_at
    assert!(harness.get_sixel_at(9, 9).is_some());
    assert!(harness.get_sixel_at(0, 0).is_none());

    // Test bounds validation
    let area = (0, 0, 80, 24);
    harness.assert_sixel_within_bounds(area)?;

    Ok(())
}
```

---

## Performance Considerations

### Query Performance

All query methods should be O(n) where n = number of Sixel graphics.

**Expected Volume**: 1-10 Sixel graphics per screen

**Benchmark Targets**:
- `sixel_count()`: < 0.1µs
- `get_sixel_at()`: < 1µs
- `assert_sixel_within_bounds()`: < 10µs
- `sequences_in_area()`: < 5µs per sequence

### Memory Usage

**SixelCapture**: ~8KB per sequence (typical)
- Vec<SixelSequence>: 8 bytes per pointer
- SixelSequence: ~40 bytes + raw data
- Raw data: 5-10 KB for typical images

**Total**: < 100 KB for typical use cases (< 10 graphics)

---

## Migration Path

### From Existing Code

**Current**:
```rust
let regions = harness.state().sixel_regions();
for region in regions {
    let within_bounds = region.start_row >= area.0
        && region.start_col >= area.1
        && /* ... complex bounds check ... */;
    assert!(within_bounds);
}
```

**New**:
```rust
let area = (5, 30, 70, 30);
harness.assert_sixel_within_bounds(area)?;
```

### Deprecation Strategy

No existing APIs are being deprecated. Phase 3 adds new methods alongside existing ones.

---

## Documentation Requirements

Each public API method must have:
1. ✅ Summary description
2. ✅ Argument documentation
3. ✅ Return value documentation
4. ✅ Error cases documented
5. ✅ At least one usage example
6. ✅ Links to related methods

---

## Success Criteria

Validation APIs are successful when:

1. ✅ All methods compile and work correctly
2. ✅ All unit tests pass (>95% coverage)
3. ✅ All integration tests pass
4. ✅ Error messages are clear and actionable
5. ✅ Examples demonstrate all methods
6. ✅ Performance targets met
7. ✅ Documentation complete
8. ✅ dgx-pixels scenarios work

---

## References

- PHASE3_CHECKLIST.md - Implementation tasks
- SIXEL_PARSING_STRATEGY.md - Parsing details
- src/sixel.rs - Existing types
- src/screen.rs - ScreenState integration

---

**Document Status**: Design Complete
**Next Step**: Begin implementation
**Maintainer**: terminal-testlib development team

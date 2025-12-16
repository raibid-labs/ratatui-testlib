# Phase 3 Test Strategy

**Version**: 1.0
**Date**: 2025-11-21
**Status**: Design Document

## Overview

This document defines the comprehensive testing strategy for Phase 3 Sixel Position Tracking. It covers unit tests, integration tests, test fixtures, and dgx-pixels validation scenarios.

---

## Testing Objectives

1. **Correctness**: Verify all Sixel parsing and tracking works accurately
2. **Coverage**: Achieve >70% code coverage for Phase 3 code
3. **Reliability**: Tests pass consistently in CI/CD
4. **Performance**: Tests complete quickly (< 5 seconds total)
5. **Maintainability**: Tests are clear and easy to update
6. **Regression**: Prevent future bugs in Sixel handling

---

## Test Pyramid

```
        ┌─────────────────┐
        │  End-to-End     │  dgx-pixels scenarios
        │  (5 tests)      │  Full workflow validation
        └─────────────────┘
              ▲
              │
        ┌─────────────────────┐
        │  Integration        │  Component interaction
        │  (15 tests)         │  Real Sixel sequences
        └─────────────────────┘
              ▲
              │
        ┌───────────────────────────┐
        │  Unit Tests               │  Individual functions
        │  (30 tests)               │  Edge cases, validation
        └───────────────────────────┘

Total: ~50 tests for Phase 3
```

---

## Test Levels

### Level 1: Unit Tests (30 tests)

**Goal**: Test individual functions and methods in isolation

**Scope**:
- Raster attribute parsing
- Pixel-to-cell conversion
- Bounds checking methods
- Position tracking helpers
- Validation logic

**Characteristics**:
- Fast (< 1ms per test)
- No I/O dependencies
- Test edge cases thoroughly
- Mock external dependencies

### Level 2: Integration Tests (15 tests)

**Goal**: Test component interactions with real data

**Scope**:
- ScreenState + Sixel detection
- VTActor + DCS callbacks
- TuiTestHarness + validation APIs
- SixelCapture + queries

**Characteristics**:
- Moderate speed (< 100ms per test)
- Uses real Sixel sequences
- Tests multiple components together
- Validates end-to-end flows

### Level 3: E2E Tests (5 tests)

**Goal**: Validate complete dgx-pixels scenarios

**Scope**:
- Gallery preview validation
- Screen transitions
- Multiple image handling
- Real-world use cases

**Characteristics**:
- Slower (< 1s per test)
- Tests complete workflows
- Validates MVP requirements
- Simulates production usage

---

## Unit Test Plan

### 1. Raster Attribute Parsing Tests

**Location**: `src/screen.rs` - `parse_raster_attributes()` tests

#### Test Cases

```rust
#[test]
fn test_parse_valid_raster_attributes() {
    let data = b"\"1;1;100;50#0~";
    let (w, h) = parse_raster_attributes(data).unwrap();
    assert_eq!(w, 100);
    assert_eq!(h, 50);
}

#[test]
fn test_parse_large_dimensions() {
    let data = b"\"2;1;5000;3000#0~";
    let (w, h) = parse_raster_attributes(data).unwrap();
    assert_eq!(w, 5000);
    assert_eq!(h, 3000);
}

#[test]
fn test_parse_missing_raster_attributes() {
    let data = b"#0;2;100;0;0#0~";  // No '"'
    let result = parse_raster_attributes(data);
    // Should use default dimensions
    assert_eq!(result, Some((100, 100)));
}

#[test]
fn test_parse_incomplete_parameters() {
    let data = b"\"1;1;100";  // Missing height
    let result = parse_raster_attributes(data);
    assert!(result.is_none() || result == Some((100, 100)));
}

#[test]
fn test_parse_malformed_numbers() {
    let data = b"\"1;1;abc;50";
    let result = parse_raster_attributes(data);
    assert!(result.is_none() || result == Some((100, 100)));
}

#[test]
fn test_parse_zero_dimensions() {
    let data = b"\"1;1;0;0";
    let result = parse_raster_attributes(data);
    // Should clamp to minimum
    assert_eq!(result, Some((1, 1)));
}

#[test]
fn test_parse_huge_dimensions() {
    let data = b"\"1;1;99999;99999";
    let (w, h) = parse_raster_attributes(data).unwrap();
    // Should clamp to maximum
    assert!(w <= 10000);
    assert!(h <= 10000);
}

#[test]
fn test_parse_with_extra_data() {
    let data = b"\"1;1;200;150#0;2;100;0;0#0~~~~~~";
    let (w, h) = parse_raster_attributes(data).unwrap();
    assert_eq!(w, 200);
    assert_eq!(h, 150);
}

#[test]
fn test_parse_with_spaces() {
    let data = b"\" 1 ; 1 ; 100 ; 50 #0";
    let (w, h) = parse_raster_attributes(data).unwrap();
    assert_eq!(w, 100);
    assert_eq!(h, 50);
}

#[test]
fn test_parse_utf8_boundary() {
    // Test with non-UTF8 bytes
    let data = b"\"1;1;100;50\xFF\xFE";
    let result = parse_raster_attributes(data);
    // Should handle gracefully
    assert!(result.is_some() || result.is_none());
}
```

**Coverage**: 10 tests, all edge cases

---

### 2. Pixel-to-Cell Conversion Tests

**Location**: `src/screen.rs` - `pixels_to_cells()` tests

```rust
#[test]
fn test_exact_division() {
    // 80 pixels / 8 = 10 cols, 60 pixels / 6 = 10 rows
    let (cols, rows) = pixels_to_cells(80, 60);
    assert_eq!(cols, 10);
    assert_eq!(rows, 10);
}

#[test]
fn test_rounding_up() {
    // 81 pixels / 8 = 10.125 → 11 cols (ceil)
    // 61 pixels / 6 = 10.167 → 11 rows (ceil)
    let (cols, rows) = pixels_to_cells(81, 61);
    assert_eq!(cols, 11);
    assert_eq!(rows, 11);
}

#[test]
fn test_minimum_dimensions() {
    let (cols, rows) = pixels_to_cells(1, 1);
    assert_eq!(cols, 1);
    assert_eq!(rows, 1);
}

#[test]
fn test_large_dimensions() {
    let (cols, rows) = pixels_to_cells(10000, 10000);
    // Should handle without overflow
    assert!(cols > 0);
    assert!(rows > 0);
}

#[test]
fn test_zero_pixels() {
    // Edge case: 0 pixels
    let (cols, rows) = pixels_to_cells(0, 0);
    // Should handle gracefully (0 or 1)
    assert!(cols <= 1);
    assert!(rows <= 1);
}

#[test]
fn test_typical_image() {
    // Typical image: 400x300 pixels
    let (cols, rows) = pixels_to_cells(400, 300);
    assert_eq!(cols, 50);  // 400/8
    assert_eq!(rows, 50);  // 300/6
}

#[test]
fn test_aspect_ratio_preserved() {
    // Wide image: 800x200
    let (cols, rows) = pixels_to_cells(800, 200);
    assert_eq!(cols, 100);  // 800/8
    assert_eq!(rows, 34);   // ceil(200/6) = 34

    let ratio = cols as f32 / rows as f32;
    assert!((ratio - 2.94).abs() < 0.1); // Approximately 3:1
}
```

**Coverage**: 7 tests, conversion accuracy validated

---

### 3. Bounds Checking Tests

**Location**: `src/screen.rs` / `src/sixel.rs` - `is_within_cells()`, `overlaps_cells()` tests

```rust
#[test]
fn test_region_completely_within() {
    let region = SixelRegion {
        start_row: 10,
        start_col: 20,
        width_cells: 10,
        height_cells: 8,
        // ... other fields
    };

    let area = (5, 15, 30, 20);  // (r=5, c=15, w=30, h=20)
    assert!(region.is_within_cells(area));
}

#[test]
fn test_region_outside() {
    let region = SixelRegion {
        start_row: 50,
        start_col: 60,
        width_cells: 10,
        height_cells: 8,
        // ...
    };

    let area = (5, 15, 30, 20);
    assert!(!region.is_within_cells(area));
}

#[test]
fn test_region_partially_outside() {
    let region = SixelRegion {
        start_row: 10,
        start_col: 20,
        width_cells: 30,  // Extends past area
        height_cells: 8,
        // ...
    };

    let area = (5, 15, 30, 20);  // Ends at col 45
    assert!(!region.is_within_cells(area));
}

#[test]
fn test_region_at_boundary() {
    let region = SixelRegion {
        start_row: 5,
        start_col: 15,
        width_cells: 30,
        height_cells: 20,
        // ...
    };

    let area = (5, 15, 30, 20);
    // Exactly matches area
    assert!(region.is_within_cells(area));
}

#[test]
fn test_overlaps_partial() {
    let region = SixelRegion {
        start_row: 10,
        start_col: 25,
        width_cells: 20,
        height_cells: 15,
        // ...
    };

    let sidebar = (0, 0, 30, 40);
    assert!(region.overlaps_cells(sidebar));
}

#[test]
fn test_overlaps_none() {
    let region = SixelRegion {
        start_row: 10,
        start_col: 50,
        width_cells: 10,
        height_cells: 8,
        // ...
    };

    let sidebar = (0, 0, 30, 40);
    assert!(!region.overlaps_cells(sidebar));
}

#[test]
fn test_zero_size_region() {
    let region = SixelRegion {
        start_row: 10,
        start_col: 20,
        width_cells: 0,
        height_cells: 0,
        // ...
    };

    let area = (5, 15, 30, 20);
    // Zero-size region should still have defined behavior
    let result = region.is_within_cells(area);
    assert!(result); // Point is within area
}

#[test]
fn test_zero_size_area() {
    let region = SixelRegion {
        start_row: 10,
        start_col: 20,
        width_cells: 10,
        height_cells: 8,
        // ...
    };

    let area = (10, 20, 0, 0);
    // Zero-size area
    let result = region.is_within_cells(area);
    assert!(!result); // Region can't fit in zero area
}
```

**Coverage**: 8 tests, all boundary conditions

---

### 4. SixelCapture Query Tests

**Location**: `src/sixel.rs` - `SixelCapture` method tests

```rust
#[test]
fn test_sequences_in_area() {
    let mut capture = SixelCapture::new();
    // Manually add test sequences...

    let area = (5, 10, 30, 20);
    let in_area = capture.sequences_in_area(area);

    assert_eq!(in_area.len(), 2);
}

#[test]
fn test_sequences_outside_area() {
    let mut capture = SixelCapture::new();
    // ...

    let area = (5, 10, 30, 20);
    let outside = capture.sequences_outside_area(area);

    assert_eq!(outside.len(), 1);
}

#[test]
fn test_bounding_box() {
    let mut capture = SixelCapture::new();
    // Add sequences at various positions...

    let bbox = capture.bounding_box().unwrap();
    assert_eq!(bbox, (5, 10, 50, 30));
}

#[test]
fn test_bounding_box_empty() {
    let capture = SixelCapture::new();
    assert!(capture.bounding_box().is_none());
}

#[test]
fn test_total_coverage() {
    let mut capture = SixelCapture::new();
    // Add sequences with known sizes...

    let coverage = capture.total_coverage();
    assert_eq!(coverage, 300); // Sum of all areas
}
```

**Coverage**: 5 tests, query methods validated

---

## Integration Test Plan

### 1. Sixel Detection Tests

**Location**: `tests/integration/sixel.rs`

```rust
#[test]
fn test_sixel_detection_basic() -> Result<()> {
    let mut screen = ScreenState::new(80, 24);

    // Feed Sixel sequence
    screen.feed(b"\x1bPq\"1;1;100;50#0~\x1b\\");

    let regions = screen.sixel_regions();
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].width, 100);
    assert_eq!(regions[0].height, 50);

    Ok(())
}

#[test]
fn test_sixel_position_tracking() -> Result<()> {
    let mut screen = ScreenState::new(80, 24);

    // Position cursor at (9, 19) [0-based]
    screen.feed(b"\x1b[10;20H");

    // Render Sixel
    screen.feed(b"\x1bPq\"1;1;100;50#0~\x1b\\");

    let regions = screen.sixel_regions();
    assert_eq!(regions[0].start_row, 9);
    assert_eq!(regions[0].start_col, 19);

    Ok(())
}

#[test]
fn test_multiple_sixel_sequences() -> Result<()> {
    let mut screen = ScreenState::new(100, 30);

    // Render 3 Sixel images at different positions
    screen.feed(b"\x1b[5;10H\x1bPq\"1;1;80;60#0~\x1b\\");
    screen.feed(b"\x1b[15;50H\x1bPq\"1;1;100;80#0~\x1b\\");
    screen.feed(b"\x1b[25;10H\x1bPq\"1;1;120;90#0~\x1b\\");

    let regions = screen.sixel_regions();
    assert_eq!(regions.len(), 3);

    // Verify positions
    assert_eq!(regions[0].start_row, 4);
    assert_eq!(regions[1].start_row, 14);
    assert_eq!(regions[2].start_row, 24);

    Ok(())
}

#[test]
fn test_sixel_with_missing_raster_attributes() -> Result<()> {
    let mut screen = ScreenState::new(80, 24);

    // Sixel without raster attributes
    screen.feed(b"\x1bPq#0;2;100;0;0#0~~~~~~\x1b\\");

    let regions = screen.sixel_regions();
    assert_eq!(regions.len(), 1);

    // Should use default dimensions
    assert_eq!(regions[0].width, 100);
    assert_eq!(regions[0].height, 100);

    Ok(())
}
```

**Coverage**: 4 tests, detection and tracking

---

### 2. Validation API Tests

**Location**: `tests/integration/sixel.rs`

```rust
#[test]
fn test_harness_assert_sixel_within_bounds() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    // Render Sixel in safe area
    harness.send_text("\x1b[10;40H")?;
    harness.send_text("\x1bPq\"1;1;400;200#0~\x1b\\")?;
    harness.update_state()?;

    // Define preview area
    let preview = (5, 30, 70, 30);

    // Should pass validation
    harness.assert_sixel_within_bounds(preview)?;

    Ok(())
}

#[test]
fn test_harness_assert_sixel_outside_bounds() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    // Render Sixel outside area
    harness.send_text("\x1b[2;10H")?;
    harness.send_text("\x1bPq\"1;1;400;200#0~\x1b\\")?;
    harness.update_state()?;

    let preview = (5, 30, 70, 30);

    // Should fail validation
    let result = harness.assert_sixel_within_bounds(preview);
    assert!(result.is_err());

    Ok(())
}

#[test]
fn test_harness_get_sixel_at() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;

    harness.send_text("\x1b[10;20H")?;
    harness.send_text("\x1bPq\"1;1;100;50#0~\x1b\\")?;
    harness.update_state()?;

    // Should find Sixel at position
    let region = harness.get_sixel_at(9, 19).unwrap();
    assert_eq!(region.width, 100);
    assert_eq!(region.height, 50);

    // Should not find at other positions
    assert!(harness.get_sixel_at(0, 0).is_none());

    Ok(())
}

#[test]
fn test_harness_sixel_count() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;

    assert_eq!(harness.sixel_count(), 0);

    // Render one Sixel
    harness.send_text("\x1bPq\"1;1;100;50#0~\x1b\\")?;
    harness.update_state()?;

    assert_eq!(harness.sixel_count(), 1);

    // Render another
    harness.send_text("\x1bPq\"1;1;100;50#0~\x1b\\")?;
    harness.update_state()?;

    assert_eq!(harness.sixel_count(), 2);

    Ok(())
}

#[test]
fn test_sixel_capture_differs_from() -> Result<()> {
    let mut screen1 = ScreenState::new(80, 24);
    screen1.feed(b"\x1bPq\"1;1;100;50#0~\x1b\\");
    let capture1 = SixelCapture::from_screen_state(&screen1);

    let screen2 = ScreenState::new(80, 24);
    let capture2 = SixelCapture::from_screen_state(&screen2);

    assert!(capture1.differs_from(&capture2));
    assert!(!capture1.differs_from(&capture1));

    Ok(())
}
```

**Coverage**: 5 tests, all validation APIs

---

### 3. Test Fixture Tests

**Location**: `tests/integration/sixel_fixtures.rs` (new file)

```rust
mod fixtures {
    // Helper to load fixtures
}

#[test]
fn test_fixture_red_100x50() -> Result<()> {
    let sixel_data = fixtures::load_sixel_fixture("red_100x50.sixel");

    let mut screen = ScreenState::new(80, 24);
    screen.feed(&sixel_data);

    let regions = screen.sixel_regions();
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].width, 100);
    assert_eq!(regions[0].height, 50);

    Ok(())
}

#[test]
fn test_fixture_blue_200x100() -> Result<()> {
    let sixel_data = fixtures::load_sixel_fixture("blue_200x100.sixel");

    let mut screen = ScreenState::new(80, 24);
    screen.feed(&sixel_data);

    let regions = screen.sixel_regions();
    assert_eq!(regions.len(), 1);
    assert_eq!(regions[0].width, 200);
    assert_eq!(regions[0].height, 100);

    Ok(())
}

#[test]
fn test_fixture_large_500x500() -> Result<()> {
    let sixel_data = fixtures::load_sixel_fixture("large_500x500.sixel");

    let mut screen = ScreenState::new(100, 50);
    screen.feed(&sixel_data);

    let regions = screen.sixel_regions();
    assert_eq!(regions.len(), 1);

    // Verify large dimensions handled
    assert_eq!(regions[0].width, 500);
    assert_eq!(regions[0].height, 500);

    Ok(())
}
```

**Coverage**: 3 tests, real fixtures validated

---

## E2E Test Plan (dgx-pixels Scenarios)

### Location: `tests/integration/dgx_pixels_scenarios.rs` (new file)

```rust
#[test]
fn test_dgx_pixels_gallery_preview() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    // Define dgx-pixels Gallery layout
    let sidebar = (0, 0, 30, 40);
    let preview = (5, 30, 70, 30);

    // Simulate image preview rendering
    harness.send_text("\x1b[10;45H")?;  // Position in preview
    harness.send_text("\x1bPq\"1;1;400;300#0~\x1b\\")?;
    harness.update_state()?;

    // Verify image in preview area
    harness.assert_sixel_within_bounds(preview)?;

    // Verify no image in sidebar
    let capture = SixelCapture::from_screen_state(&harness.state());
    assert!(capture.sequences_in_area(sidebar).is_empty());

    Ok(())
}

#[test]
fn test_dgx_pixels_screen_transition() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    // Gallery with image
    harness.send_text("\x1bPq\"1;1;400;300#0~\x1b\\")?;
    harness.update_state()?;
    assert_eq!(harness.sixel_count(), 1);

    let before = SixelCapture::from_screen_state(&harness.state());

    // Clear screen (transition)
    harness.send_text("\x1b[2J")?;
    harness.update_state()?;

    let after = SixelCapture::from_screen_state(&harness.state());

    // Note: Clearing behavior depends on implementation
    // This test verifies the capture can detect changes
    assert!(after.differs_from(&before));

    Ok(())
}

#[test]
fn test_dgx_pixels_multiple_thumbnails() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    // Render 3 thumbnails in grid
    let positions = [(10, 35), (10, 55), (10, 75)];

    for (row, col) in &positions {
        harness.send_text(&format!("\x1b[{};{}H", row + 1, col + 1))?;
        harness.send_text("\x1bPq\"1;1;80;60#0~\x1b\\")?;
    }
    harness.update_state()?;

    assert_eq!(harness.sixel_count(), 3);

    // Verify each thumbnail position
    for (row, col) in &positions {
        let region = harness.get_sixel_at(*row, *col);
        assert!(region.is_some(), "Expected Sixel at ({}, {})", row, col);
    }

    // Verify all within preview area
    let preview = (5, 30, 70, 30);
    harness.assert_sixel_within_bounds(preview)?;

    Ok(())
}

#[test]
fn test_dgx_pixels_image_replaced() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    // Render first image
    harness.send_text("\x1b[10;40H\x1bPq\"1;1;200;150#0~\x1b\\")?;
    harness.update_state()?;
    let first = SixelCapture::from_screen_state(&harness.state());

    // Render second image at same position
    harness.send_text("\x1b[10;40H\x1bPq\"1;1;300;200#0~\x1b\\")?;
    harness.update_state()?;
    let second = SixelCapture::from_screen_state(&harness.state());

    // Captures should differ
    assert!(second.differs_from(&first));

    // Second image should have different dimensions
    // (Implementation detail: may track both or just latest)
    assert!(harness.sixel_count() >= 1);

    Ok(())
}

#[test]
fn test_dgx_pixels_preview_boundary() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    let preview = (5, 30, 70, 30);

    // Render image exactly at preview boundary
    harness.send_text("\x1b[6;31H")?;  // (5, 30) in 0-based
    harness.send_text("\x1bPq\"1;1;560;180#0~\x1b\\")?;  // 70x30 cells
    harness.update_state()?;

    // Should be within bounds
    harness.assert_sixel_within_bounds(preview)?;

    Ok(())
}
```

**Coverage**: 5 tests, full dgx-pixels scenarios

---

## Test Fixtures

### Directory Structure

```
tests/fixtures/sixel/
├── README.md              # Documentation
├── minimal_10x10.sixel    # Minimal test (10x10 pixels)
├── red_100x50.sixel       # Solid red (100x50 pixels)
├── blue_200x100.sixel     # Solid blue (200x100 pixels)
├── gradient_150x150.sixel # Gradient pattern (150x150 pixels)
└── large_500x500.sixel    # Large image (500x500 pixels)
```

### Fixture README.md

```markdown
# Sixel Test Fixtures

This directory contains Sixel graphics test data for terminal-testlib Phase 3.

## Fixtures

### minimal_10x10.sixel
- Dimensions: 10x10 pixels
- Content: Solid color
- Use: Basic parsing tests

### red_100x50.sixel
- Dimensions: 100x50 pixels
- Content: Solid red (#FF0000)
- Use: Position tracking tests

### blue_200x100.sixel
- Dimensions: 200x100 pixels
- Content: Solid blue (#0000FF)
- Use: Bounds checking tests

### gradient_150x150.sixel
- Dimensions: 150x150 pixels
- Content: Color gradient
- Use: Color palette tests

### large_500x500.sixel
- Dimensions: 500x500 pixels
- Content: Test pattern
- Use: Performance tests

## Usage

```rust
use tests::fixtures;

let sixel_data = fixtures::load_sixel_fixture("red_100x50.sixel");
screen.feed(&sixel_data);
```

## Generating Fixtures

Use `img2sixel` or similar tools:

```bash
convert -size 100x50 xc:red red_100x50.png
img2sixel red_100x50.png > red_100x50.sixel
```
```

### Fixture Helper Module

**Location**: `tests/helpers/sixel_fixtures.rs` (new file)

```rust
use std::path::PathBuf;

pub fn sixel_fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("sixel")
        .join(name)
}

pub fn load_sixel_fixture(name: &str) -> Vec<u8> {
    std::fs::read(sixel_fixture_path(name))
        .unwrap_or_else(|e| panic!("Failed to load fixture {}: {}", name, e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_fixture() {
        let data = load_sixel_fixture("red_100x50.sixel");
        assert!(!data.is_empty());
        assert!(data.starts_with(b"\x1bP"));  // DCS start
    }
}
```

---

## CI/CD Integration

### GitHub Actions Configuration

```yaml
# .github/workflows/ci.yml

name: CI

on: [push, pull_request]

jobs:
  test-phase3:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Run Phase 3 unit tests
        run: cargo test --lib sixel

      - name: Run Phase 3 integration tests
        run: |
          cargo test --test sixel
          cargo test --test dgx_pixels_scenarios

      - name: Check test performance
        run: |
          time cargo test --lib sixel -- --nocapture
          # Should complete in < 5 seconds

      - name: Code coverage
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir coverage
          # Target: >70% coverage for Phase 3 code
```

---

## Coverage Goals

### Phase 3 Coverage Targets

| Component | Target Coverage | Priority |
|-----------|----------------|----------|
| parse_raster_attributes() | 100% | P0 |
| pixels_to_cells() | 100% | P0 |
| SixelRegion methods | 90% | P0 |
| Harness validation APIs | 85% | P0 |
| SixelCapture queries | 80% | P1 |
| Integration flows | 75% | P1 |
| Overall Phase 3 | >70% | P0 |

### Coverage Tools

```bash
# Local coverage check
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/tarpaulin-report.html

# CI coverage
cargo tarpaulin --out Xml --output-dir coverage
```

---

## Performance Benchmarks

### Benchmark Tests

**Location**: `benches/sixel_benchmarks.rs` (new file)

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use term_test::*;

fn bench_parse_raster_attributes(c: &mut Criterion) {
    let data = b"\"1;1;100;50#0~";

    c.bench_function("parse_raster_attributes", |b| {
        b.iter(|| {
            // Benchmark parsing
            let result = parse_raster_attributes(black_box(data));
            black_box(result);
        });
    });
}

fn bench_pixels_to_cells(c: &mut Criterion) {
    c.bench_function("pixels_to_cells", |b| {
        b.iter(|| {
            let result = pixels_to_cells(black_box(400), black_box(300));
            black_box(result);
        });
    });
}

fn bench_bounds_checking(c: &mut Criterion) {
    let region = SixelRegion { /* ... */ };
    let area = (5, 30, 70, 30);

    c.bench_function("is_within_cells", |b| {
        b.iter(|| {
            let result = region.is_within_cells(black_box(area));
            black_box(result);
        });
    });
}

criterion_group!(
    benches,
    bench_parse_raster_attributes,
    bench_pixels_to_cells,
    bench_bounds_checking
);
criterion_main!(benches);
```

### Performance Targets

| Operation | Target | Acceptable |
|-----------|--------|------------|
| parse_raster_attributes() | < 1µs | < 10µs |
| pixels_to_cells() | < 0.1µs | < 1µs |
| is_within_cells() | < 0.1µs | < 1µs |
| assert_sixel_within_bounds() | < 10µs | < 100µs |
| Full test suite | < 3s | < 5s |

---

## Test Maintenance

### Test Review Checklist

- [ ] All tests have descriptive names
- [ ] Each test tests one thing
- [ ] Tests are independent (no ordering dependencies)
- [ ] Tests clean up resources
- [ ] Timeouts are reasonable
- [ ] Error messages are clear
- [ ] Tests are documented with comments

### Refactoring Guidelines

**When to refactor tests**:
1. Test fails intermittently (flaky)
2. Test is slow (> 100ms for unit test)
3. Test is unclear or complex
4. Test has code duplication
5. Test requirements change

**How to refactor**:
1. Identify the problem
2. Extract common setup into helper
3. Use fixtures for test data
4. Simplify assertions
5. Add comments for complex logic

---

## Success Criteria

Test strategy is successful when:

1. ✅ All unit tests pass (30+ tests)
2. ✅ All integration tests pass (15+ tests)
3. ✅ All E2E tests pass (5+ tests)
4. ✅ Code coverage >70% for Phase 3
5. ✅ Tests complete in < 5 seconds
6. ✅ No flaky tests in CI/CD
7. ✅ All fixtures load correctly
8. ✅ Benchmarks meet targets
9. ✅ Tests are maintainable
10. ✅ Documentation is clear

---

## References

- PHASE3_CHECKLIST.md - Implementation tasks
- SIXEL_PARSING_STRATEGY.md - Parsing details
- PHASE3_VALIDATION_API.md - API specification
- tests/integration/sixel.rs - Existing tests

---

**Document Status**: Design Complete
**Next Step**: Begin implementation
**Maintainer**: terminal-testlib development team

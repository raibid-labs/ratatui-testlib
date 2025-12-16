# Sixel Parsing Strategy

**Version**: 1.0
**Date**: 2025-11-21
**Status**: Design Document

## Overview

This document describes the parsing strategy for Sixel graphics in terminal-testlib, focusing on extracting dimensions and position information needed for dgx-pixels testing.

---

## Sixel Escape Sequence Format

### Complete Sequence Structure

```
DCS Pa ; Pad ; Ph ; Pv q <sixel_data> ST

Where:
- DCS = Device Control String = ESC P (0x1b 0x50)
- Pa = Pixel aspect ratio (1 = 5:1, 2 = 3:1, default 2)
- Pad = Background color mode (1 = keep background, 2 = set background)
- Ph = Horizontal pixel count (image width)
- Pv = Vertical pixel count (image height)
- q = Sixel mode identifier (0x71)
- <sixel_data> = Color definitions and raster data
- ST = String Terminator = ESC \ (0x1b 0x5c)
```

### Raster Attribute Format

The raster attributes appear in the Sixel data as:

```
" Pa ; Pad ; Ph ; Pv
```

**Example**:
```
ESC P q " 1 ; 1 ; 100 ; 50 # 0 ; 2 ; 100 ; 0 ; 0 # 0 ~~~~~~...~ ESC \
        ↑
        Raster attributes: aspect=1, pad=1, width=100px, height=50px
```

---

## Current Implementation

### Location
`src/screen.rs:120-144` - `parse_raster_attributes()` method

### Current Code
```rust
fn parse_raster_attributes(&self, data: &[u8]) -> Option<(u32, u32)> {
    let data_str = std::str::from_utf8(data).ok()?;

    // Find the raster attributes command starting with '"'
    if let Some(raster_start) = data_str.find('"') {
        let raster_part = &data_str[raster_start + 1..];

        // Parse format: Pa;Pb;Ph;Pv
        // We want Ph (width) and Pv (height)
        let parts: Vec<&str> = raster_part
            .split(|c: char| !c.is_ascii_digit() && c != ';')
            .filter(|s| !s.is_empty())
            .take(4)
            .collect();

        if parts.len() >= 4 {
            let width = parts[2].parse::<u32>().ok()?;
            let height = parts[3].parse::<u32>().ok()?;
            return Some((width, height));
        }
    }
    None
}
```

### Current Strengths ✅
- Correctly identifies raster attributes with '"' marker
- Parses semicolon-separated parameters
- Filters out non-numeric characters
- Returns None for invalid sequences

### Current Limitations 🔶
- Returns None instead of fallback dimensions
- Doesn't handle missing raster attributes
- Doesn't validate dimension ranges
- Doesn't estimate dimensions from data
- No logging for debugging

---

## Parsing Strategy

### Phase 1: Extract Raster Attributes

**Goal**: Extract Ph (width) and Pv (height) from raster attribute string

**Algorithm**:
1. Convert Sixel data bytes to UTF-8 string
2. Search for '"' character (raster attribute marker)
3. Extract substring after '"'
4. Split by semicolon to get 4 parameters: [Pa, Pad, Ph, Pv]
5. Parse Ph (index 2) and Pv (index 3) as u32
6. Validate ranges (0 < dim < 10000)
7. Return (width, height)

**Edge Cases**:
- No '"' marker → Use fallback estimation
- Incomplete parameters (< 4) → Use defaults
- Malformed numbers → Skip invalid params
- Zero dimensions → Use minimum (1x1)
- Huge dimensions (> 10000) → Clamp to max

### Phase 2: Fallback Dimension Estimation

**Goal**: Provide reasonable dimensions when raster attributes are missing

**Strategy 1: Default Dimensions**
- If no raster attributes: use 100x100 as default
- Simple, predictable, works for basic tests
- **Recommended for MVP**

**Strategy 2: Data-based Estimation** (Future Enhancement)
- Analyze Sixel data size
- Estimate rows from newline count
- Estimate columns from repeat sequences
- More accurate but complex

**Implementation**:
```rust
fn parse_raster_attributes(&self, data: &[u8]) -> Option<(u32, u32)> {
    // Try to parse raster attributes
    if let Some((w, h)) = self.try_parse_raster_attributes(data) {
        return Some((w, h));
    }

    // Fallback: use default dimensions
    Some((100, 100))  // Default 100x100 pixels
}
```

### Phase 3: Dimension Validation

**Goal**: Ensure parsed dimensions are reasonable

**Validation Rules**:
1. **Minimum**: width ≥ 1, height ≥ 1
2. **Maximum**: width ≤ 10000, height ≤ 10000
3. **Sanity**: log warning if dimensions > 2000
4. **Clamping**: clamp out-of-range values

**Implementation**:
```rust
fn validate_dimensions(width: u32, height: u32) -> (u32, u32) {
    let w = width.clamp(1, 10000);
    let h = height.clamp(1, 10000);

    if w > 2000 || h > 2000 {
        eprintln!("Warning: Large Sixel dimensions: {}x{}", w, h);
    }

    (w, h)
}
```

---

## Pixel-to-Cell Conversion

### Terminal Cell Dimensions

Terminals display graphics in character cells. Need to convert pixel dimensions to cell counts.

**Standard Ratios**:
- **Vertical**: 6 pixels per line (Sixel standard)
- **Horizontal**: 8-10 pixels per column (font-dependent)

**Common Configurations**:
| Terminal | H pixels/col | V pixels/row | Notes |
|----------|-------------|--------------|-------|
| xterm    | 8           | 6            | Common |
| mlterm   | 10          | 6            | Wider |
| WezTerm  | 8           | 6            | Standard |
| kitty    | 10          | 6            | Configurable |

### Conversion Algorithm

**Formula**:
```
cell_width = ceiling(pixel_width / h_pixels_per_cell)
cell_height = ceiling(pixel_height / v_pixels_per_cell)
```

**Implementation**:
```rust
impl TerminalState {
    // Configuration (defaults)
    const V_PIXELS_PER_CELL: u32 = 6;  // Sixel standard
    const H_PIXELS_PER_CELL: u32 = 8;  // Common default

    fn pixels_to_cells(&self, width_px: u32, height_px: u32) -> (u16, u16) {
        let cols = ((width_px + Self::H_PIXELS_PER_CELL - 1)
                    / Self::H_PIXELS_PER_CELL) as u16;
        let rows = ((height_px + Self::V_PIXELS_PER_CELL - 1)
                    / Self::V_PIXELS_PER_CELL) as u16;
        (cols, rows)
    }
}
```

**Example**:
```rust
// Sixel: 100x50 pixels
// Terminal: 8 pixels/col, 6 pixels/row
// Cells: ceiling(100/8) x ceiling(50/6) = 13 x 9 cells
```

### Rounding Strategy

**Use Ceiling (Round Up)**:
- Ensures Sixel never truncated
- Bounds checking covers full graphic
- Conservative approach for testing

**Alternative**: Floor (Round Down)
- Would underestimate size
- Could miss overflow bugs
- Not recommended for bounds checking

---

## Position Tracking

### Cursor Position Capture

**Current Implementation** (src/screen.rs:173-214):
```rust
fn dcs_hook(&mut self, mode: u8, params: &[i64], ...) {
    if mode == b'q' {  // Sixel mode
        self.in_sixel_mode = true;
        // Cursor position at this point is the Sixel start position
        // Will be captured in dcs_unhook()
    }
}

fn dcs_unhook(&mut self) {
    if self.in_sixel_mode {
        let (width, height) = self.parse_raster_attributes(...);
        let (width_cells, height_cells) = self.pixels_to_cells(width, height);

        let region = SixelRegion {
            start_row: self.cursor_pos.0,  // Captured here
            start_col: self.cursor_pos.1,  // Captured here
            width,
            height,
            width_cells,
            height_cells,
            data: self.current_sixel_data.clone(),
        };
        self.sixel_regions.push(region);
    }
}
```

**Position Semantics**:
- Position is where Sixel **starts** rendering
- Captured from cursor position at DCS start
- 0-based row/column indexing
- Independent of terminal size (can be out of bounds)

### Coordinate System

**Terminal Coordinates**:
```
     0   1   2   3   ...  78  79
   ┌───┬───┬───┬───┬───┬───┬───┐
 0 │   │   │   │   │   │   │   │  Row 0
   ├───┼───┼───┼───┼───┼───┼───┤
 1 │   │   │   │   │   │   │   │  Row 1
   ├───┼───┼───┼───┼───┼───┼───┤
   ...
   │   │ X │   │   │   │   │   │  Sixel at (5, 10)
   │   │   │   │   │   │   │   │
```

**Bounds Representation**:
```rust
type Area = (row: u16, col: u16, width: u16, height: u16);

// Example: Preview area occupies rows 5-34, cols 30-99
let preview_area = (5, 30, 70, 30);
//                  ↑   ↑   ↑   ↑
//               row col  w   h
```

---

## Bounds Checking Strategy

### Containment Check

**Definition**: Sixel is "within bounds" if:
- start_row ≥ area_row
- start_col ≥ area_col
- start_row + height_cells ≤ area_row + area_height
- start_col + width_cells ≤ area_col + area_width

**Implementation**:
```rust
impl SixelRegion {
    pub fn is_within_cells(&self, area: (u16, u16, u16, u16)) -> bool {
        let (area_row, area_col, area_width, area_height) = area;

        self.start_row >= area_row
            && self.start_col >= area_col
            && (self.start_row + self.height_cells) <= (area_row + area_height)
            && (self.start_col + self.width_cells) <= (area_col + area_width)
    }
}
```

**Visual Example**:
```
Area: (5, 30, 70, 30)  ← Preview panel
Region: (10, 40, 25x15 cells) ← Image

         30              40         65         100
    5    ┌────────────────────────────────────┐
         │                                    │
   10    │          ┌──────────┐              │
         │          │  Image   │              │
         │          │  25x15   │              │
   25    │          └──────────┘              │
         │                                    │
   35    └────────────────────────────────────┘

Calculation:
- start_row (10) ≥ area_row (5) ✓
- start_col (40) ≥ area_col (30) ✓
- start_row + height (10+15=25) ≤ area_row + area_height (5+30=35) ✓
- start_col + width (40+25=65) ≤ area_col + area_width (30+70=100) ✓

Result: Image is within bounds ✓
```

### Overlap Check

**Definition**: Sixel "overlaps" area if any part intersects

**Implementation**:
```rust
pub fn overlaps_cells(&self, area: (u16, u16, u16, u16)) -> bool {
    let (area_row, area_col, area_width, area_height) = area;

    // Check if rectangles don't overlap, then negate
    !(self.start_row + self.height_cells <= area_row
        || self.start_col + self.width_cells <= area_col
        || self.start_row >= area_row + area_height
        || self.start_col >= area_col + area_width)
}
```

**Use Cases**:
- Detect graphics in forbidden areas
- Find graphics near boundaries
- Check for partial overlaps

---

## Test Data Strategy

### Fixture Types

**1. Minimal Sixel** (for parsing tests):
```
ESC P q " 1 ; 1 ; 10 ; 10 # 0 ESC \
```
- 10x10 pixels
- No actual raster data
- Tests parser only

**2. Solid Color Rectangle** (for visual tests):
```
ESC P q " 1 ; 1 ; 100 ; 50 # 0 ; 2 ; 100 ; 0 ; 0 # 0 ~~~...~~~ ESC \
```
- 100x50 pixels
- Red color (#RGB = 100,0,0)
- Filled with color 0
- Easy to verify dimensions

**3. Small Image** (for integration tests):
```
ESC P q " 1 ; 1 ; 80 ; 60 <actual_sixel_data> ESC \
```
- 80x60 pixels (10x10 cells)
- Real image data
- Validates full pipeline

### Test Fixture Organization

```
tests/fixtures/sixel/
├── README.md              # Documentation
├── minimal_10x10.sixel    # Minimal test
├── red_100x50.sixel       # Solid color
├── blue_200x100.sixel     # Larger solid
├── gradient_150x150.sixel # Pattern test
└── large_500x500.sixel    # Performance test
```

### Fixture Helper Functions

```rust
#[cfg(test)]
pub mod fixtures {
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
            .unwrap_or_else(|_| panic!("Failed to load {}", name))
    }

    // Generate programmatic Sixel for testing
    pub fn generate_solid_sixel(width: u32, height: u32) -> Vec<u8> {
        format!(
            "\x1bPq\"1;1;{};{}#0;2;100;0;0#0",
            width, height
        )
        .as_bytes()
        .to_vec()
    }
}
```

---

## Error Handling Strategy

### Parse Errors

**Approach**: Graceful degradation

**Scenarios**:
1. **No raster attributes** → Use default (100x100)
2. **Incomplete parameters** → Use partial data + defaults
3. **Invalid numbers** → Skip, use defaults
4. **Out of range** → Clamp to valid range
5. **UTF-8 errors** → Return None

**Implementation**:
```rust
fn parse_raster_attributes(&self, data: &[u8]) -> Option<(u32, u32)> {
    // Try parsing
    match self.try_parse_raster_attributes(data) {
        Some((w, h)) => {
            // Validate and clamp
            let (w, h) = self.validate_dimensions(w, h);
            Some((w, h))
        }
        None => {
            // Fallback to defaults
            eprintln!("Warning: Could not parse Sixel raster attributes, using defaults");
            Some((100, 100))
        }
    }
}
```

### Validation Errors

**Approach**: Clear error messages with context

**Example**:
```rust
pub fn assert_sixel_within_bounds(&self, area: (u16, u16, u16, u16)) -> Result<()> {
    let capture = SixelCapture::from_screen_state(&self.state);
    let outside = capture.sequences_outside_area(area);

    if !outside.is_empty() {
        let positions: Vec<_> = outside.iter()
            .map(|s| format!("({}, {})", s.position.0, s.position.1))
            .collect();

        return Err(TermTestError::SixelValidation(format!(
            "Found {} Sixel sequence(s) outside area {:?}:\n  Positions: {}",
            outside.len(),
            area,
            positions.join(", ")
        )));
    }

    Ok(())
}
```

**Error Message Example**:
```
Error: SixelValidation
  Found 2 Sixel sequence(s) outside area (5, 30, 70, 30):
  Positions: (2, 10), (40, 95)
```

---

## Performance Considerations

### Parsing Performance

**Expected Volume**: Low (1-10 Sixel sequences per screen)

**Optimization Strategy**:
- Parse once during DCS processing
- Cache parsed dimensions in SixelRegion
- Avoid re-parsing for queries

**Benchmark Targets**:
- Parse raster attributes: < 1µs
- Pixel-to-cell conversion: < 0.1µs
- Bounds checking: < 0.1µs

### Memory Usage

**Sixel Data Storage**:
- Store raw Sixel data for debugging
- Compress if > 1MB (future)
- Option to disable data storage

**Typical Sizes**:
- Small image (100x100): ~5-10 KB
- Medium image (500x500): ~100-200 KB
- Large image (1000x1000): ~500 KB - 1 MB

---

## Testing Strategy

### Unit Tests (src/screen.rs)

**Parse Tests**:
```rust
#[test]
fn test_parse_valid_raster_attributes() {
    let data = b"\"1;1;100;50#0~";
    let (w, h) = parse_raster_attributes(data).unwrap();
    assert_eq!(w, 100);
    assert_eq!(h, 50);
}

#[test]
fn test_parse_missing_raster_attributes() {
    let data = b"#0~";  // No raster attributes
    let (w, h) = parse_raster_attributes(data).unwrap();
    // Should use defaults
    assert_eq!(w, 100);
    assert_eq!(h, 100);
}
```

**Conversion Tests**:
```rust
#[test]
fn test_pixels_to_cells() {
    // 100x60 pixels → 13x10 cells (8px/col, 6px/row)
    let (cols, rows) = pixels_to_cells(100, 60);
    assert_eq!(cols, 13); // ceil(100/8)
    assert_eq!(rows, 10); // ceil(60/6)
}
```

### Integration Tests (tests/integration/sixel.rs)

**Position Tracking**:
```rust
#[test]
fn test_sixel_position_tracking() -> Result<()> {
    let mut screen = ScreenState::new(80, 24);

    screen.feed(b"\x1b[10;20H");  // Move to (9, 19) 0-based
    screen.feed(b"\x1bPq\"1;1;100;50#0~\x1b\\");

    let regions = screen.sixel_regions();
    assert_eq!(regions[0].start_row, 9);
    assert_eq!(regions[0].start_col, 19);
    Ok(())
}
```

**Bounds Validation**:
```rust
#[test]
fn test_sixel_bounds_validation() -> Result<()> {
    let mut harness = TuiTestHarness::new(120, 40)?;

    // Render in preview area
    harness.send_text("\x1b[10;40H")?;
    harness.send_text("\x1bPq\"1;1;400;200#0~\x1b\\")?;
    harness.update_state()?;

    let preview_area = (5, 30, 70, 30);
    harness.assert_sixel_within_bounds(preview_area)?;

    Ok(())
}
```

---

## Implementation Phases

### Phase 1: Enhance Parsing (Days 1-2)
1. Improve parse_raster_attributes()
2. Add fallback defaults
3. Add dimension validation
4. Write unit tests

### Phase 2: Add Conversion (Days 3-4)
1. Implement pixels_to_cells()
2. Update SixelRegion struct
3. Add conversion tests
4. Update dcs_unhook()

### Phase 3: Bounds Checking (Days 5-6)
1. Add is_within_cells() method
2. Add overlaps_cells() method
3. Write unit tests
4. Add integration tests

### Phase 4: Validation APIs (Days 7-8)
1. Add harness methods
2. Enhance SixelCapture
3. Write integration tests
4. Test with fixtures

### Phase 5: Documentation (Days 9-10)
1. Complete rustdoc
2. Write user guide
3. Create examples
4. Update README

---

## Success Criteria

Parsing strategy is successful when:

1. ✅ Raster attributes parsed correctly (>95% accuracy)
2. ✅ Missing attributes handled gracefully (fallbacks work)
3. ✅ Dimensions validated and clamped appropriately
4. ✅ Pixel-to-cell conversion accurate (within 1 cell)
5. ✅ Bounds checking detects overflow correctly
6. ✅ Position tracking verified accurate
7. ✅ All unit tests pass (>20 parsing tests)
8. ✅ All integration tests pass
9. ✅ Performance acceptable (< 10µs per parse)
10. ✅ Documentation complete with examples

---

## Future Enhancements

### Phase 3+
- Support for color palette extraction
- Sixel data compression
- Progressive rendering detection
- Animation frame tracking

### Post-MVP
- Advanced dimension estimation from data
- Per-terminal pixel ratio configuration
- Sixel replay/rendering for debugging
- Visual regression testing

---

## References

- DEC Sixel Graphics Specification: https://www.vt100.net/docs/vt3xx-gp/chapter14.html
- vtparse documentation: https://docs.rs/vtparse/
- libsixel reference: https://github.com/saitoha/libsixel
- Terminal graphics testing research: docs/sixel-research.md

---

**Document Status**: Design Complete
**Next Step**: Begin implementation (PHASE3_CHECKLIST.md)
**Maintainer**: terminal-testlib development team

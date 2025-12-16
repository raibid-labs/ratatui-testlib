# Grid State Verification API - Issue #8

## Overview

This document describes the new grid state verification API added to terminal-testlib to enable terminal emulator testing and comparison. The API exposes the internal screen grid state, making it possible to verify that another terminal emulator matches terminal-testlib's behavior cell-by-cell.

## Problem Statement

When testing terminal emulators (like Scarab), we need to:
1. Feed the same ANSI escape sequences to both implementations
2. Compare the final grid state cell-by-cell
3. Verify character content, colors, and text attributes match

Previously, the internal grid/cell structure was difficult to access from external crates.

## Solution

The new API provides multiple ways to inspect screen state:

### 1. Dimension Accessors

```rust
let screen = ScreenState::new(80, 24);

// Get individual dimensions
let width = screen.cols();   // 80
let height = screen.rows();  // 24

// Get both dimensions
let (width, height) = screen.size();  // (80, 24)
```

### 2. Cell Access

Access individual cells with full attribute information:

```rust
let cell = screen.get_cell(row, col).unwrap();

// Cell has public fields:
println!("Char: {}", cell.c);
println!("Foreground: {:?}", cell.fg);  // Option<u8>
println!("Background: {:?}", cell.bg);  // Option<u8>
println!("Bold: {}", cell.bold);
println!("Italic: {}", cell.italic);
println!("Underline: {}", cell.underline);
```

### 3. Row Iteration

Iterate over all rows:

```rust
for (row_idx, row) in screen.iter_rows().enumerate() {
    for (col_idx, cell) in row.iter().enumerate() {
        println!("Cell ({}, {}): '{}'", row_idx, col_idx, cell.c);
    }
}
```

Iterate over a specific row:

```rust
if let Some(cells) = screen.iter_row(0) {
    for (col, cell) in cells.enumerate() {
        // Process cells in row 0
    }
}
```

### 4. Grid Snapshot

Capture the complete screen state for deep comparison:

```rust
let snapshot = screen.snapshot();

// GridSnapshot contains:
// - width: u16
// - height: u16
// - cells: Vec<Vec<Cell>>
// - cursor: (u16, u16)

// Direct cell access
let cell = &snapshot.cells[row][col];

// Compare snapshots
let snapshot1 = screen1.snapshot();
let snapshot2 = screen2.snapshot();
assert_eq!(snapshot1, snapshot2);
```

## API Reference

### New Methods on `ScreenState`

| Method | Description | Returns |
|--------|-------------|---------|
| `cols()` | Screen width in columns | `u16` |
| `rows()` | Screen height in rows | `u16` |
| `iter_rows()` | Iterator over all rows | `impl Iterator<Item = &[Cell]>` |
| `iter_row(row)` | Iterator over cells in specific row | `Option<impl Iterator<Item = &Cell>>` |
| `snapshot()` | Complete grid state capture | `GridSnapshot` |

### Existing Methods (Now Documented for Verification)

| Method | Description | Returns |
|--------|-------------|---------|
| `get_cell(row, col)` | Access individual cell | `Option<&Cell>` |
| `size()` | Screen dimensions | `(u16, u16)` |
| `cursor_position()` | Current cursor position | `(u16, u16)` |

### Public Types

#### `Cell`

```rust
pub struct Cell {
    pub c: char,              // Character
    pub fg: Option<u8>,       // Foreground color (0-255 or None)
    pub bg: Option<u8>,       // Background color (0-255 or None)
    pub bold: bool,           // Bold attribute
    pub italic: bool,         // Italic attribute
    pub underline: bool,      // Underline attribute
}
```

#### `GridSnapshot`

```rust
pub struct GridSnapshot {
    pub width: u16,                  // Screen width in columns
    pub height: u16,                 // Screen height in rows
    pub cells: Vec<Vec<Cell>>,       // Complete grid (row-major)
    pub cursor: (u16, u16),          // Cursor position (row, col)
}
```

## Use Cases

### 1. Verification Oracle

Use terminal-testlib as a reference implementation:

```rust
use terminal_testlib::ScreenState;

// Define test sequence
let test_sequence = b"\x1b[31mRed\x1b[32mGreen\x1b[34mBlue";

// Create oracle
let mut oracle = ScreenState::new(80, 24);
oracle.feed(test_sequence);

// Feed to system-under-test
let mut sut = YourTerminalEmulator::new(80, 24);
sut.feed(test_sequence);

// Compare cell-by-cell
for row in 0..oracle.rows() {
    for col in 0..oracle.cols() {
        let oracle_cell = oracle.get_cell(row, col).unwrap();
        let sut_cell = sut.get_cell(row, col).unwrap();

        assert_eq!(oracle_cell.c, sut_cell.c);
        assert_eq!(oracle_cell.fg, sut_cell.fg);
        assert_eq!(oracle_cell.bg, sut_cell.bg);
        assert_eq!(oracle_cell.bold, sut_cell.bold);
        assert_eq!(oracle_cell.italic, sut_cell.italic);
        assert_eq!(oracle_cell.underline, sut_cell.underline);
    }
}
```

### 2. Snapshot Testing

Compare complete grid states:

```rust
let snapshot1 = oracle.snapshot();
let snapshot2 = sut.snapshot();

// Direct comparison
assert_eq!(snapshot1, snapshot2);

// Or compare specific aspects
assert_eq!(snapshot1.cursor, snapshot2.cursor);
assert_eq!(snapshot1.cells[0], snapshot2.cells[0]); // First row
```

### 3. Regression Testing

Store snapshots for regression testing:

```rust
// Capture golden snapshot
let golden = oracle.snapshot();

// Later, verify new implementation matches
let current = sut.snapshot();
assert_eq!(current, golden, "Implementation changed behavior");
```

## Examples

### Complete Example: Terminal Emulator Comparison

See `examples/grid_verification.rs` for a comprehensive demonstration.

### Quick Example: Color Verification

```rust
use terminal_testlib::ScreenState;

let mut screen = ScreenState::new(80, 24);
screen.feed(b"\x1b[31mRed\x1b[0m");

// Verify red text
let cell = screen.get_cell(0, 0).unwrap();
assert_eq!(cell.c, 'R');
assert_eq!(cell.fg, Some(1)); // ANSI red

// Verify reset
let cell = screen.get_cell(0, 3).unwrap();
assert_eq!(cell.fg, None); // Default
```

## Testing

Comprehensive test suite in `tests/grid_verification.rs`:

- Dimension accessors
- Cell access with all attributes
- 256-color mode support
- Row iteration
- Grid snapshots
- Snapshot comparison
- Out-of-bounds handling
- Verification oracle patterns

Run tests:

```bash
cargo test --test grid_verification
```

## Performance Considerations

1. **Cell Access**: O(1) for `get_cell(row, col)`
2. **Row Iteration**: Zero-copy iteration over existing data
3. **Snapshot**: O(n) clone of entire grid - use when deep comparison needed
4. **Iterator**: Lazy evaluation - only processes cells you actually access

## Backward Compatibility

All changes are additive - no breaking changes to existing API:

- Existing methods unchanged
- New methods added
- Cell struct fields were already public
- GridSnapshot is a new type

## Future Enhancements

Potential additions (not in current scope):

1. Partial snapshots (specific regions)
2. Diff between snapshots
3. Serialization support (serde)
4. Custom comparison functions
5. Performance optimizations for large grids

## References

- Issue: #8 - Expose Screen/Grid state for verification
- Example: `examples/grid_verification.rs`
- Tests: `tests/grid_verification.rs`
- Module: `src/screen.rs`

## Acceptance Criteria

All requirements from issue #8 are met:

- ✅ `screen.get_cell(col, row)` accessor exists
- ✅ Cell struct exposes: character, foreground, background, attributes
- ✅ `screen.rows()` and `screen.cols()` dimension accessors
- ✅ Iterator support for rows/cells
- ✅ `screen.snapshot()` for structured export
- ✅ Public exports in lib.rs
- ✅ Documentation with examples
- ✅ Comprehensive tests (23 tests passing)

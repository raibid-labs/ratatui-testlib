# Stream-Based Parsing API

This document describes how to use terminal-testlib as a headless terminal emulator parser without any PTY overhead. This is particularly useful for:

- **Terminal emulator testing**: Use terminal-testlib as a reference implementation/verification oracle
- **Escape sequence validation**: Test ANSI/VT100 sequence parsing behavior
- **Integration testing**: Feed deterministic byte sequences and verify output
- **Performance testing**: Zero PTY overhead for high-throughput parsing

## Quick Start

```rust
use terminal_testlib::ScreenState;

// Create a parser without any PTY
let mut screen = ScreenState::new(80, 24);

// Feed raw byte sequences
let input = b"\x1b[31mHello, World!\x1b[0m";
screen.feed(input);

// Query the parsed state
assert!(screen.contains("Hello, World!"));
assert_eq!(screen.get_cell(0, 0).unwrap().fg, Some(1)); // Red color
assert_eq!(screen.cursor_position(), (0, 13));
```

## Core API

### Creating a Parser

```rust
use terminal_testlib::ScreenState;

// Standard constructor
let mut screen = ScreenState::new(80, 24);

// Or use the Parser type alias for clarity
use terminal_testlib::Parser;
let mut parser = Parser::new(80, 24);
```

### Feeding Data

The `feed()` method processes raw bytes through the VT100/ANSI parser:

```rust
// Feed a complete sequence
screen.feed(b"\x1b[31mRed Text\x1b[0m");

// Feed incrementally (parser maintains state across calls)
screen.feed(b"\x1b[3");  // Partial ESC sequence
screen.feed(b"1m");       // Complete the sequence
screen.feed(b"Text");     // Regular text
```

### Querying State

#### Text Content

```rust
// Get all screen contents
let contents = screen.contents(); // Returns String with newlines

// Get specific row
let row = screen.row_contents(5); // Row 5 (0-indexed)

// Get specific character
let ch = screen.text_at(0, 0); // Returns Option<char>

// Search for text
assert!(screen.contains("Welcome"));
```

#### Cursor Position

```rust
let (row, col) = screen.cursor_position(); // Both 0-indexed
assert_eq!(row, 5);
assert_eq!(col, 10);
```

#### Cell Attributes

```rust
use terminal_testlib::Cell;

if let Some(cell) = screen.get_cell(0, 0) {
    println!("Character: {}", cell.c);
    println!("Foreground: {:?}", cell.fg);   // Option<u8>
    println!("Background: {:?}", cell.bg);   // Option<u8>
    println!("Bold: {}", cell.bold);
    println!("Italic: {}", cell.italic);
    println!("Underline: {}", cell.underline);
}
```

#### Sixel Graphics

```rust
// Get all Sixel regions
let regions = screen.sixel_regions();
for region in regions {
    println!("Sixel at ({}, {}), size {}x{} pixels",
        region.start_row, region.start_col,
        region.width, region.height);
}

// Check specific position
if screen.has_sixel_at(10, 20) {
    println!("Sixel graphics at (10, 20)");
}
```

## Use Cases

### 1. Verification Oracle Pattern

Use ScreenState as a reference implementation to verify another terminal emulator:

```rust
use terminal_testlib::ScreenState;

// Define test sequence
let test_seq = b"\x1b[2J\x1b[H\x1b[31mTest\x1b[0m";

// Create oracle
let mut oracle = ScreenState::new(80, 24);
oracle.feed(test_seq);

// Feed same sequence to your system-under-test
// Then compare:
// - oracle.contents() vs sut.contents()
// - oracle.cursor_position() vs sut.cursor_position()
// - oracle.get_cell(r, c) vs sut.get_cell(r, c)
```

### 2. ANSI Sequence Testing

Test specific ANSI escape sequence behaviors:

```rust
use terminal_testlib::ScreenState;

#[test]
fn test_cursor_movement() {
    let mut screen = ScreenState::new(80, 24);

    // Move to (10, 20) - 1-based in CSI
    screen.feed(b"\x1b[10;20H");
    assert_eq!(screen.cursor_position(), (9, 19));

    // Move forward 5 columns
    screen.feed(b"\x1b[5C");
    assert_eq!(screen.cursor_position(), (9, 24));
}
```

### 3. Color and Attribute Testing

```rust
#[test]
fn test_sgr_sequences() {
    let mut screen = ScreenState::new(80, 24);

    // Bold + Red
    screen.feed(b"\x1b[1;31mBold Red\x1b[0m");

    let cell = screen.get_cell(0, 0).unwrap();
    assert_eq!(cell.c, 'B');
    assert_eq!(cell.fg, Some(1)); // Red
    assert!(cell.bold);

    // After reset
    let cell = screen.get_cell(0, 8).unwrap();
    assert!(!cell.bold);
}
```

### 4. Incremental Parsing

Test that the parser handles partial sequences correctly:

```rust
#[test]
fn test_partial_sequences() {
    let mut screen = ScreenState::new(80, 24);

    // Feed in parts
    screen.feed(b"\x1b");      // ESC
    screen.feed(b"[");         // CSI
    screen.feed(b"3");         // param
    screen.feed(b"1");         // param
    screen.feed(b"m");         // final
    screen.feed(b"Red");       // text

    assert!(screen.contains("Red"));
    assert_eq!(screen.get_cell(0, 0).unwrap().fg, Some(1));
}
```

### 5. Sixel Graphics Testing

```rust
#[test]
fn test_sixel_positioning() {
    let mut screen = ScreenState::new(80, 24);

    // Position cursor and render Sixel
    screen.feed(b"\x1b[10;20H");          // Move cursor
    screen.feed(b"\x1bPq");                // Start Sixel
    screen.feed(b"\"1;1;100;50");          // Raster: 100x50px
    screen.feed(b"#0;2;100;100;100");      // Color
    screen.feed(b"#0~");                   // Data
    screen.feed(b"\x1b\\");                // End

    let regions = screen.sixel_regions();
    assert_eq!(regions.len(), 1);

    let region = &regions[0];
    assert_eq!(region.start_row, 9);   // 0-based
    assert_eq!(region.start_col, 19);  // 0-based
    assert_eq!(region.width, 100);
    assert_eq!(region.height, 50);
}
```

### 6. Multiple Independent Parsers

You can create multiple independent parser instances:

```rust
let mut parser1 = ScreenState::new(80, 24);
let mut parser2 = ScreenState::new(100, 30);

parser1.feed(b"Parser 1");
parser2.feed(b"Parser 2");

assert!(parser1.contains("Parser 1"));
assert!(!parser1.contains("Parser 2"));
```

## Supported Escape Sequences

### Cursor Movement

- `ESC [ H` / `ESC [ {row} ; {col} H` - Cursor Position (CUP)
- `ESC [ {n} A` - Cursor Up (CUU)
- `ESC [ {n} B` - Cursor Down (CUD)
- `ESC [ {n} C` - Cursor Forward (CUF)
- `ESC [ {n} D` - Cursor Backward (CUB)
- `ESC D` - Index (IND)
- `ESC E` - Next Line (NEL)

### Text Attributes (SGR)

- `ESC [ 0 m` - Reset all attributes
- `ESC [ 1 m` - Bold
- `ESC [ 3 m` - Italic
- `ESC [ 4 m` - Underline
- `ESC [ 22 m` - Normal intensity (not bold)
- `ESC [ 23 m` - Not italic
- `ESC [ 24 m` - Not underlined

### Colors

**Basic Colors (30-37, 40-47)**:
- `ESC [ 30-37 m` - Foreground colors
- `ESC [ 40-47 m` - Background colors
- `ESC [ 39 m` - Default foreground
- `ESC [ 49 m` - Default background

**Bright Colors (90-97, 100-107)**:
- `ESC [ 90-97 m` - Bright foreground
- `ESC [ 100-107 m` - Bright background

**256-Color Mode**:
- `ESC [ 38 ; 5 ; {n} m` - Foreground (n = 0-255)
- `ESC [ 48 ; 5 ; {n} m` - Background (n = 0-255)

### Graphics

**Sixel**:
- `ESC P q {data} ESC \` - Sixel graphics sequence
- Raster attributes: `" {pan} ; {pad} ; {width} ; {height}`
- Tracks position, width, height, and raw data

### Control Characters

- `\r` - Carriage return
- `\n` - Line feed
- `\t` - Tab (advances to next 8-column boundary)

## Performance Considerations

### Zero PTY Overhead

ScreenState parsing has zero PTY overhead:

```rust
// No process spawning, no PTY allocation
let mut screen = ScreenState::new(80, 24);
screen.feed(data); // Direct parsing
```

### Incremental Processing

The parser maintains state across `feed()` calls:

```rust
// These are equivalent but allow streaming:
screen.feed(b"\x1b[31mHello\x1b[0m");

// vs

screen.feed(b"\x1b[31m");
screen.feed(b"Hello");
screen.feed(b"\x1b[0m");
```

### Memory Usage

Memory usage is proportional to screen size:

```rust
// 80x24 screen = 1,920 cells
let screen = ScreenState::new(80, 24);

// Each Cell is ~8 bytes (char + attributes)
// Total: ~15KB plus parser state
```

## Comparison with PTY-Based Testing

| Feature | Stream-Based (`ScreenState`) | PTY-Based (`TuiTestHarness`) |
|---------|------------------------------|------------------------------|
| Process spawning | No | Yes |
| PTY allocation | No | Yes |
| Deterministic | Yes | Yes |
| Input simulation | Manual byte sequences | Keyboard events |
| Use case | Parser testing, oracles | Full TUI app testing |
| Performance | Fastest | Slower (PTY overhead) |
| Setup complexity | Minimal | Higher |

## Examples

See the following for complete examples:

- [`examples/stream_parsing.rs`](../examples/stream_parsing.rs) - Comprehensive examples
- [`tests/stream_parsing.rs`](../tests/stream_parsing.rs) - Test suite demonstrating all features

## API Reference

For detailed API documentation, see:

```bash
cargo doc --open
```

Then navigate to `terminal_testlib::ScreenState`.

## Contributing

When adding new escape sequence support:

1. Add parsing logic to `src/screen.rs` (VTActor trait)
2. Add tests to `src/screen.rs` (unit tests)
3. Add integration tests to `tests/stream_parsing.rs`
4. Update this documentation
5. Run `cargo test` and `cargo doc`

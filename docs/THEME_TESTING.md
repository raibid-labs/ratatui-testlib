# Theme and Color Palette Verification

This document describes the theme verification functionality added to `terminal-testlib` for testing terminal color palettes and themes.

## Overview

The `theme` module provides utilities for verifying that terminals correctly apply color palettes and themes. This is particularly useful for:

- Testing custom terminal emulator themes
- Verifying color scheme implementations
- Ensuring consistent color rendering across platforms
- Integration testing for terminal UI applications with custom theming

## Features

### Standard ANSI Color Support

The `AnsiColor` enum represents the 16 standard terminal colors (0-15):

```rust
pub enum AnsiColor {
    Black = 0,
    Red = 1,
    Green = 2,
    Yellow = 3,
    Blue = 4,
    Magenta = 5,
    Cyan = 6,
    White = 7,
    BrightBlack = 8,
    BrightRed = 9,
    BrightGreen = 10,
    BrightYellow = 11,
    BrightBlue = 12,
    BrightMagenta = 13,
    BrightCyan = 14,
    BrightWhite = 15,
}
```

### Built-in Color Palettes

The module includes four popular terminal themes:

1. **Slime** - Scarab's default theme with green accents
2. **Dracula** - Popular dark theme with purple accents
3. **Nord** - Arctic, north-bluish color palette
4. **Monokai** - Classic syntax highlighting theme

Each palette includes:
- 16 ANSI colors as RGBA values
- Default background color
- Default foreground color
- Cursor color
- Selection highlight color

### Color Verification Trait

The `ThemeTestExt` trait provides methods for verifying colors in terminal output:

```rust
pub trait ThemeTestExt {
    /// Get cell attributes at a specific position
    fn cell_attrs_at(&self, row: u16, col: u16) -> IpcResult<CellAttributes>;

    /// Get the foreground color at a cell
    fn cell_foreground(&self, row: u16, col: u16) -> IpcResult<u32>;

    /// Get the background color at a cell
    fn cell_background(&self, row: u16, col: u16) -> IpcResult<u32>;

    /// Assert foreground color matches expected
    fn assert_foreground_color(&self, row: u16, col: u16, expected: u32) -> IpcResult<()>;

    /// Assert background color matches expected
    fn assert_background_color(&self, row: u16, col: u16, expected: u32) -> IpcResult<()>;

    /// Assert ANSI color matches palette
    fn assert_ansi_color(
        &self,
        row: u16,
        col: u16,
        palette: &ColorPalette,
        expected_ansi: AnsiColor,
    ) -> IpcResult<()>;

    /// Capture cell colors for snapshot testing
    fn capture_cell_colors(&self, row: u16, col: u16) -> IpcResult<(u32, u32)>;

    /// Scan a region for all unique colors
    fn scan_colors_in_region(
        &self,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
    ) -> IpcResult<ColorScan>;
}
```

## Usage Examples

### Basic Color Verification

```rust
use terminal_testlib::{
    scarab::ScarabTestHarness,
    theme::{AnsiColor, ColorPalette, ThemeTestExt},
};
use std::time::Duration;

let mut harness = ScarabTestHarness::connect()?;

// Send colored text
harness.send_input("\x1b[31mRed Text\x1b[0m\n")?;
harness.wait_for_text("Red Text", Duration::from_secs(2))?;

// Verify the color using a palette
let palette = ColorPalette::dracula();
harness.assert_ansi_color(0, 0, &palette, AnsiColor::Red)?;
```

### Scanning Colors in a Region

```rust
// Scan a 10x20 region for all unique colors
let scan = harness.scan_colors_in_region(0, 0, 10, 20)?;

println!("Found {} unique foreground colors", scan.unique_foreground_count());
println!("Found {} unique background colors", scan.unique_background_count());

for color in &scan.foreground_colors {
    println!("Foreground: 0x{:08X}", color);
}
```

### Custom Palette Testing

```rust
// Create a custom palette
let mut palette = ColorPalette::new("custom");
palette.colors[AnsiColor::Red.as_index() as usize] = 0xFF000080;
palette.background = 0x1E1E1EFF;
palette.foreground = 0xD4D4D4FF;

// Test against custom palette
harness.assert_ansi_color(5, 10, &palette, AnsiColor::Red)?;
```

### Approximate Color Matching

For terminals that apply color correction or gamma adjustments:

```rust
let palette = ColorPalette::slime();
let fg = harness.cell_foreground(0, 0)?;

// Match with tolerance of 10 units per channel
if palette.matches_ansi_approx(fg, AnsiColor::Green, 10) {
    println!("Color matches green (approximately)");
}
```

### Snapshot Testing

```rust
// Capture colors for snapshot comparison
let (fg, bg) = harness.capture_cell_colors(5, 10)?;

// Save to snapshot for regression testing
assert_eq!(fg, expected_foreground);
assert_eq!(bg, expected_background);
```

## Implementation Details

### Color Format

All colors are stored as 32-bit RGBA values in the format:

```
0xRRGGBBAA
```

Where:
- RR = Red channel (0-255)
- GG = Green channel (0-255)
- BB = Blue channel (0-255)
- AA = Alpha channel (0-255)

### Alpha Channel Handling

When comparing colors, the alpha channel is typically ignored (using mask `0xFFFFFF00`) since most terminal emulators don't support transparency in text rendering.

### CellAttributes Structure

Cell attributes are read from shared memory with this structure:

```rust
#[repr(C)]
pub struct CellAttributes {
    pub fg: u32,           // Foreground RGBA
    pub bg: u32,           // Background RGBA
    pub flags: u16,        // Style flags
    pub reserved: u16,     // Reserved for future use
}
```

Style flags include:
- Bold
- Italic
- Underline
- Strikethrough
- Reverse video

## Integration with Scarab

The `ScarabTestHarness` implements `ThemeTestExt` automatically when the `ipc` feature is enabled. This allows seamless theme verification:

```rust
use terminal_testlib::{
    scarab::ScarabTestHarness,
    theme::{ColorPalette, ThemeTestExt},
};

let mut harness = ScarabTestHarness::connect()?;

// Use theme verification methods directly
let fg = harness.cell_foreground(0, 0)?;
let palette = ColorPalette::slime();
println!("Using slime theme: 0x{:08X}", palette.foreground);
```

## Examples

See the following examples for more usage patterns:

- `examples/theme_test.rs` - Comprehensive demonstration of theme verification
- `tests/theme_tests.rs` - Unit tests showing all functionality

## Running Examples

```bash
# Enable Scarab test mode
export SCARAB_TEST_RTL=1

# Run the theme test example
cargo run --example theme_test --features scarab
```

## Feature Flags

The theme module requires the `ipc` feature:

```toml
[dependencies]
terminal-testlib = { version = "0.4", features = ["ipc"] }
```

For Scarab-specific testing:

```toml
[dependencies]
terminal-testlib = { version = "0.4", features = ["scarab"] }
```

## Testing Your Own Themes

To test a custom terminal theme:

1. Configure your terminal to use the theme
2. Create a `ColorPalette` matching your theme colors
3. Send ANSI escape sequences to render colored text
4. Use `assert_ansi_color` to verify the colors match
5. Use `scan_colors_in_region` to analyze rendered output

Example:

```rust
// Define your theme
let my_theme = ColorPalette::new("my_theme");
// ... set colors ...

// Test red text
harness.send_input("\x1b[31mRed\x1b[0m")?;
harness.wait_for_text("Red", Duration::from_secs(1))?;
harness.assert_ansi_color(0, 0, &my_theme, AnsiColor::Red)?;
```

## Troubleshooting

### Colors Don't Match

If color assertions fail:

1. Verify your terminal supports the color mode (256-color, true color)
2. Check if color correction is applied by the terminal
3. Use `matches_ansi_approx` with tolerance instead of exact matching
4. Capture actual colors with `cell_foreground/cell_background` to see what's rendered

### Cell Attributes Not Available

If `cell_attrs_at` returns default values:

1. Ensure the daemon exposes cell attributes in shared memory
2. Check that `attrs_offset` and `attrs_size` are set in the shared memory header
3. Verify the shared memory layout matches `CellAttributes` structure

### Out of Bounds Errors

If you get "out of bounds" errors:

1. Check terminal dimensions with `harness.dimensions()`
2. Ensure coordinates are 0-indexed
3. Account for scrolling or viewport changes

## API Reference

See the module documentation for complete API details:

```bash
cargo doc --features scarab --open
```

Navigate to `terminal_testlib::theme` for full documentation.

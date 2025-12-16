//! Integration tests for theme and color palette verification.

#![cfg(feature = "ipc")]

use terminal_testlib::{
    ipc::{CellAttributes, IpcResult},
    theme::{AnsiColor, ColorPalette, ColorScan, ThemeTestExt},
};

/// Mock implementation for testing ThemeTestExt
struct MockTerminal {
    cells: Vec<Vec<CellAttributes>>,
}

impl MockTerminal {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            cells: vec![vec![CellAttributes::default(); cols]; rows],
        }
    }

    fn set_cell(&mut self, row: usize, col: usize, attrs: CellAttributes) {
        self.cells[row][col] = attrs;
    }
}

impl ThemeTestExt for MockTerminal {
    fn cell_attrs_at(&self, row: u16, col: u16) -> IpcResult<CellAttributes> {
        let row = row as usize;
        let col = col as usize;

        if row >= self.cells.len() || col >= self.cells[row].len() {
            return Err(terminal_testlib::ipc::IpcError::InvalidData(
                "Out of bounds".to_string(),
            ));
        }

        Ok(self.cells[row][col])
    }
}

#[test]
fn test_ansi_color_conversions() {
    assert_eq!(AnsiColor::Black.as_index(), 0);
    assert_eq!(AnsiColor::Red.as_index(), 1);
    assert_eq!(AnsiColor::BrightWhite.as_index(), 15);

    assert_eq!(AnsiColor::from_index(0), Some(AnsiColor::Black));
    assert_eq!(AnsiColor::from_index(15), Some(AnsiColor::BrightWhite));
    assert_eq!(AnsiColor::from_index(16), None);
}

#[test]
fn test_color_palette_slime() {
    let palette = ColorPalette::slime();
    assert_eq!(palette.name, "slime");

    // Check signature green (slime) color
    let green = palette.ansi_color(AnsiColor::Green);
    assert_eq!(green, 0x4ECDC4FF);

    // Check bright green (bright slime)
    let bright_green = palette.ansi_color(AnsiColor::BrightGreen);
    assert_eq!(bright_green, 0x52FFB8FF);

    // Verify background and foreground
    assert_eq!(palette.background, 0x0D1117FF);
    assert_eq!(palette.foreground, 0xECF0F1FF);
}

#[test]
fn test_color_palette_dracula() {
    let palette = ColorPalette::dracula();
    assert_eq!(palette.name, "dracula");

    // Check signature purple/blue
    let blue = palette.ansi_color(AnsiColor::Blue);
    assert_eq!(blue, 0xBD93F9FF);

    // Check pink
    let magenta = palette.ansi_color(AnsiColor::Magenta);
    assert_eq!(magenta, 0xFF79C6FF);

    // Verify background and foreground
    assert_eq!(palette.background, 0x282A36FF);
    assert_eq!(palette.foreground, 0xF8F8F2FF);
}

#[test]
fn test_color_palette_nord() {
    let palette = ColorPalette::nord();
    assert_eq!(palette.name, "nord");

    // Check nord blue
    let blue = palette.ansi_color(AnsiColor::Blue);
    assert_eq!(blue, 0x81A1C1FF);

    // Verify background and foreground
    assert_eq!(palette.background, 0x2E3440FF);
    assert_eq!(palette.foreground, 0xD8DEE9FF);
}

#[test]
fn test_color_palette_monokai() {
    let palette = ColorPalette::monokai();
    assert_eq!(palette.name, "monokai");

    // Check signature pink/red
    let red = palette.ansi_color(AnsiColor::Red);
    assert_eq!(red, 0xF92672FF);

    // Check green
    let green = palette.ansi_color(AnsiColor::Green);
    assert_eq!(green, 0xA6E22EFF);

    // Verify background and foreground
    assert_eq!(palette.background, 0x272822FF);
    assert_eq!(palette.foreground, 0xF8F8F2FF);
}

#[test]
fn test_palette_matches_ansi() {
    let palette = ColorPalette::slime();

    // Test exact match
    let green = palette.ansi_color(AnsiColor::Green);
    assert!(palette.matches_ansi(green, AnsiColor::Green));
    assert!(!palette.matches_ansi(green, AnsiColor::Red));

    // Test alpha channel is ignored
    let green_diff_alpha = (green & 0xFFFFFF00) | 0x80;
    assert!(palette.matches_ansi(green_diff_alpha, AnsiColor::Green));
}

#[test]
fn test_palette_matches_ansi_approx() {
    let palette = ColorPalette::slime();
    let green = palette.ansi_color(AnsiColor::Green);

    // Exact match with zero tolerance
    assert!(palette.matches_ansi_approx(green, AnsiColor::Green, 0));

    // Create slightly different color
    let r = ((green >> 24) & 0xFF) as u8;
    let g = ((green >> 16) & 0xFF) as u8;
    let b = ((green >> 8) & 0xFF) as u8;

    let similar = ((r.wrapping_add(5) as u32) << 24)
        | ((g as u32) << 16)
        | ((b as u32) << 8)
        | 0xFF;

    // Should match with tolerance of 10
    assert!(palette.matches_ansi_approx(similar, AnsiColor::Green, 10));

    // Should not match with tolerance of 2
    assert!(!palette.matches_ansi_approx(similar, AnsiColor::Green, 2));
}

#[test]
fn test_color_scan_basic() {
    let mut scan = ColorScan::new();

    assert_eq!(scan.cells_scanned, 0);
    assert_eq!(scan.unique_foreground_count(), 0);
    assert_eq!(scan.unique_background_count(), 0);

    scan.add_foreground(0xFF0000FF);
    scan.add_foreground(0x00FF00FF);
    scan.add_background(0x000000FF);
    scan.increment_cells();

    assert_eq!(scan.cells_scanned, 1);
    assert_eq!(scan.unique_foreground_count(), 2);
    assert_eq!(scan.unique_background_count(), 1);
}

#[test]
fn test_color_scan_duplicates() {
    let mut scan = ColorScan::new();

    scan.add_foreground(0xFF0000FF);
    scan.add_foreground(0xFF0000FF); // Duplicate
    scan.add_foreground(0x00FF00FF);

    assert_eq!(scan.unique_foreground_count(), 2);
    assert!(scan.foreground_colors.contains(&0xFF0000FF));
    assert!(scan.foreground_colors.contains(&0x00FF00FF));
}

#[test]
fn test_theme_test_ext_cell_colors() {
    let mut terminal = MockTerminal::new(10, 10);

    // Set up a cell with red foreground and black background
    let attrs = CellAttributes {
        fg: 0xFF0000FF,
        bg: 0x000000FF,
        flags: 0,
        reserved: 0,
    };
    terminal.set_cell(0, 0, attrs);

    // Test cell_foreground
    let fg = terminal.cell_foreground(0, 0).unwrap();
    assert_eq!(fg, 0xFF0000FF);

    // Test cell_background
    let bg = terminal.cell_background(0, 0).unwrap();
    assert_eq!(bg, 0x000000FF);

    // Test capture_cell_colors
    let (captured_fg, captured_bg) = terminal.capture_cell_colors(0, 0).unwrap();
    assert_eq!(captured_fg, 0xFF0000FF);
    assert_eq!(captured_bg, 0x000000FF);
}

#[test]
fn test_theme_test_ext_assert_colors() {
    let mut terminal = MockTerminal::new(10, 10);

    let attrs = CellAttributes {
        fg: 0xFF0000FF,
        bg: 0x000000FF,
        flags: 0,
        reserved: 0,
    };
    terminal.set_cell(0, 0, attrs);

    // Test successful assertions (ignoring alpha)
    assert!(terminal
        .assert_foreground_color(0, 0, 0xFF0000FF)
        .is_ok());
    assert!(terminal
        .assert_foreground_color(0, 0, 0xFF000080)
        .is_ok()); // Different alpha
    assert!(terminal.assert_background_color(0, 0, 0x000000FF).is_ok());

    // Test failed assertions
    assert!(terminal
        .assert_foreground_color(0, 0, 0x00FF00FF)
        .is_err());
    assert!(terminal.assert_background_color(0, 0, 0xFF0000FF).is_err());
}

#[test]
fn test_theme_test_ext_assert_ansi_color() {
    let mut terminal = MockTerminal::new(10, 10);
    let palette = ColorPalette::slime();

    // Set cell to green from slime palette
    let green = palette.ansi_color(AnsiColor::Green);
    let attrs = CellAttributes {
        fg: green,
        bg: 0x000000FF,
        flags: 0,
        reserved: 0,
    };
    terminal.set_cell(5, 5, attrs);

    // Should match
    assert!(terminal
        .assert_ansi_color(5, 5, &palette, AnsiColor::Green)
        .is_ok());

    // Should not match other colors
    assert!(terminal
        .assert_ansi_color(5, 5, &palette, AnsiColor::Red)
        .is_err());
}

#[test]
fn test_theme_test_ext_scan_region() {
    let mut terminal = MockTerminal::new(10, 10);

    // Set up a 3x3 region with different colors
    let colors = [
        0xFF0000FF, // Red
        0x00FF00FF, // Green
        0x0000FFFF, // Blue
    ];

    for row in 0..3 {
        for col in 0..3 {
            let attrs = CellAttributes {
                fg: colors[(row + col) % 3],
                bg: 0x000000FF,
                flags: 0,
                reserved: 0,
            };
            terminal.set_cell(row, col, attrs);
        }
    }

    // Scan the region
    let scan = terminal.scan_colors_in_region(0, 0, 2, 2).unwrap();

    assert_eq!(scan.cells_scanned, 9);
    assert_eq!(scan.unique_foreground_count(), 3);
    assert_eq!(scan.unique_background_count(), 1);
    assert!(scan.foreground_colors.contains(&0xFF0000FF));
    assert!(scan.foreground_colors.contains(&0x00FF00FF));
    assert!(scan.foreground_colors.contains(&0x0000FFFF));
}

#[test]
fn test_theme_test_ext_scan_region_out_of_bounds() {
    let terminal = MockTerminal::new(5, 5);

    // Scanning beyond bounds should gracefully skip invalid cells
    let scan = terminal.scan_colors_in_region(0, 0, 10, 10).unwrap();

    // Should only scan the valid 5x5 region
    assert!(scan.cells_scanned <= 25);
}

#[test]
fn test_all_palettes_have_complete_data() {
    let palettes = vec![
        ColorPalette::slime(),
        ColorPalette::dracula(),
        ColorPalette::nord(),
        ColorPalette::monokai(),
    ];

    for palette in palettes {
        // Check all 16 colors exist
        assert_eq!(palette.colors.len(), 16);

        // Check special colors are set (not default black)
        assert_ne!(palette.background, 0);
        assert_ne!(palette.foreground, 0);
        assert_ne!(palette.cursor, 0);
        assert_ne!(palette.selection, 0);

        // Check all ANSI colors can be retrieved
        for i in 0..16 {
            let ansi = AnsiColor::from_index(i).unwrap();
            let color = palette.ansi_color(ansi);
            assert_ne!(color, 0);
        }
    }
}

#[test]
fn test_custom_palette() {
    let mut palette = ColorPalette::new("custom");

    assert_eq!(palette.name, "custom");

    // Customize a color
    palette.colors[AnsiColor::Red.as_index() as usize] = 0xFF000080;

    let red = palette.ansi_color(AnsiColor::Red);
    assert_eq!(red, 0xFF000080);

    // Test matching
    assert!(palette.matches_ansi(0xFF0000FF, AnsiColor::Red)); // Alpha ignored
}

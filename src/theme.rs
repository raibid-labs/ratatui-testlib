//! Theme and color palette verification helpers.
//!
//! This module provides utilities for verifying that terminals correctly apply
//! themes and color palettes. It includes:
//!
//! - Standard ANSI color indices and common terminal themes
//! - Color palette definitions with support for popular themes
//! - Extension traits for verifying colors at specific cell positions
//! - Color scanning utilities for analyzing terminal output
//!
//! # Quick Start
//!
//! ```rust,no_run
//! # #[cfg(feature = "scarab")]
//! # {
//! use std::time::Duration;
//! use terminal_testlib::scarab::ScarabTestHarness;
//! use terminal_testlib::theme::{ColorPalette, AnsiColor, ThemeTestExt};
//!
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! let mut harness = ScarabTestHarness::connect()?;
//!
//! // Send colored text
//! harness.send_input("\x1b[31mRed Text\x1b[0m\n")?;
//! harness.wait_for_text("Red Text", Duration::from_secs(2))?;
//!
//! // Verify the color using a palette
//! let palette = ColorPalette::dracula();
//! harness.assert_ansi_color(0, 0, &palette, AnsiColor::Red)?;
//!
//! // Scan colors in a region
//! let scan = harness.scan_colors_in_region(0, 0, 5, 20)?;
//! println!("Found {} unique foreground colors", scan.foreground_colors.len());
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! # Supported Themes
//!
//! Built-in theme palettes include:
//! - **Slime**: Scarab's default theme (green-focused)
//! - **Dracula**: Popular dark theme with purple accents
//! - **Nord**: Arctic, north-bluish color palette
//! - **Monokai**: Classic syntax highlighting theme

use crate::ipc::{CellAttributes, IpcError, IpcResult};
use std::time::Duration;

/// Standard ANSI color indices.
///
/// These map to the 16 standard terminal colors (0-15) that
/// can be customized by terminal themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum AnsiColor {
    /// ANSI color 0: Black
    Black = 0,
    /// ANSI color 1: Red
    Red = 1,
    /// ANSI color 2: Green
    Green = 2,
    /// ANSI color 3: Yellow
    Yellow = 3,
    /// ANSI color 4: Blue
    Blue = 4,
    /// ANSI color 5: Magenta
    Magenta = 5,
    /// ANSI color 6: Cyan
    Cyan = 6,
    /// ANSI color 7: White
    White = 7,
    /// ANSI color 8: Bright Black (Gray)
    BrightBlack = 8,
    /// ANSI color 9: Bright Red
    BrightRed = 9,
    /// ANSI color 10: Bright Green
    BrightGreen = 10,
    /// ANSI color 11: Bright Yellow
    BrightYellow = 11,
    /// ANSI color 12: Bright Blue
    BrightBlue = 12,
    /// ANSI color 13: Bright Magenta
    BrightMagenta = 13,
    /// ANSI color 14: Bright Cyan
    BrightCyan = 14,
    /// ANSI color 15: Bright White
    BrightWhite = 15,
}

impl AnsiColor {
    /// Convert to index (0-15).
    pub fn as_index(self) -> u8 {
        self as u8
    }

    /// Create from index, returns None if out of range.
    pub fn from_index(index: u8) -> Option<Self> {
        match index {
            0 => Some(AnsiColor::Black),
            1 => Some(AnsiColor::Red),
            2 => Some(AnsiColor::Green),
            3 => Some(AnsiColor::Yellow),
            4 => Some(AnsiColor::Blue),
            5 => Some(AnsiColor::Magenta),
            6 => Some(AnsiColor::Cyan),
            7 => Some(AnsiColor::White),
            8 => Some(AnsiColor::BrightBlack),
            9 => Some(AnsiColor::BrightRed),
            10 => Some(AnsiColor::BrightGreen),
            11 => Some(AnsiColor::BrightYellow),
            12 => Some(AnsiColor::BrightBlue),
            13 => Some(AnsiColor::BrightMagenta),
            14 => Some(AnsiColor::BrightCyan),
            15 => Some(AnsiColor::BrightWhite),
            _ => None,
        }
    }
}

/// Color palette definition.
///
/// Stores the 16 ANSI colors plus special terminal colors
/// (background, foreground, cursor, selection) as 32-bit RGBA values.
///
/// The format is 0xRRGGBBAA where:
/// - RR = Red channel (0-255)
/// - GG = Green channel (0-255)
/// - BB = Blue channel (0-255)
/// - AA = Alpha channel (0-255, usually 0xFF)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorPalette {
    /// Name of the palette/theme.
    pub name: String,
    /// The 16 ANSI colors as RGBA values.
    pub colors: [u32; 16],
    /// Default background color.
    pub background: u32,
    /// Default foreground color.
    pub foreground: u32,
    /// Cursor color.
    pub cursor: u32,
    /// Selection highlight color.
    pub selection: u32,
}

impl ColorPalette {
    /// Create a new empty palette with default black/white colors.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            colors: [0xFF; 16], // All white by default
            background: 0x000000FF, // Black
            foreground: 0xFFFFFFFF, // White
            cursor: 0xFFFFFFFF,     // White
            selection: 0x444444FF,  // Dark gray
        }
    }

    /// Create the "slime" theme palette (Scarab default).
    ///
    /// A green-focused theme with bright accents, optimized for
    /// terminal work and code highlighting.
    pub fn slime() -> Self {
        Self {
            name: "slime".to_string(),
            colors: [
                0x0D1117FF, // Black
                0xFF6B6BFF, // Red
                0x4ECDC4FF, // Green (slime green)
                0xF7DC6FFF, // Yellow
                0x5DADE2FF, // Blue
                0xC39BD3FF, // Magenta
                0x48C9B0FF, // Cyan
                0xECF0F1FF, // White
                0x34495EFF, // Bright Black
                0xFF7979FF, // Bright Red
                0x52FFB8FF, // Bright Green (bright slime)
                0xFFEB3BFF, // Bright Yellow
                0x74B9FFFF, // Bright Blue
                0xE1BEE7FF, // Bright Magenta
                0x76FF03FF, // Bright Cyan (lime)
                0xFFFFFFFF, // Bright White
            ],
            background: 0x0D1117FF,
            foreground: 0xECF0F1FF,
            cursor: 0x52FFB8FF,
            selection: 0x264F78FF,
        }
    }

    /// Create the "dracula" theme palette.
    ///
    /// Popular dark theme with purple accents and high contrast.
    /// Source: https://draculatheme.com
    pub fn dracula() -> Self {
        Self {
            name: "dracula".to_string(),
            colors: [
                0x282A36FF, // Black
                0xFF5555FF, // Red
                0x50FA7BFF, // Green
                0xF1FA8CFF, // Yellow
                0xBD93F9FF, // Blue (purple-ish)
                0xFF79C6FF, // Magenta
                0x8BE9FDFF, // Cyan
                0xF8F8F2FF, // White
                0x6272A4FF, // Bright Black
                0xFF6E6EFF, // Bright Red
                0x69FF94FF, // Bright Green
                0xFFFFA5FF, // Bright Yellow
                0xD6ACFFFF, // Bright Blue
                0xFF92DFFF, // Bright Magenta
                0xA4FFFFEF, // Bright Cyan
                0xFFFFFFFF, // Bright White
            ],
            background: 0x282A36FF,
            foreground: 0xF8F8F2FF,
            cursor: 0xF8F8F2FF,
            selection: 0x44475AFF,
        }
    }

    /// Create the "nord" theme palette.
    ///
    /// Arctic, north-bluish color palette with muted tones.
    /// Source: https://www.nordtheme.com
    pub fn nord() -> Self {
        Self {
            name: "nord".to_string(),
            colors: [
                0x3B4252FF, // Black
                0xBF616AFF, // Red
                0xA3BE8CFF, // Green
                0xEBCB8BFF, // Yellow
                0x81A1C1FF, // Blue
                0xB48EADFF, // Magenta
                0x88C0D0FF, // Cyan
                0xE5E9F0FF, // White
                0x4C566AFF, // Bright Black
                0xD08770FF, // Bright Red
                0x8FBCBBFF, // Bright Green (cyan-ish)
                0xEBCB8BFF, // Bright Yellow
                0x5E81ACFF, // Bright Blue
                0xB48EADFF, // Bright Magenta
                0x8FBCBBFF, // Bright Cyan
                0xECEFF4FF, // Bright White
            ],
            background: 0x2E3440FF,
            foreground: 0xD8DEE9FF,
            cursor: 0xD8DEE9FF,
            selection: 0x434C5EFF,
        }
    }

    /// Create the "monokai" theme palette.
    ///
    /// Classic syntax highlighting theme with vibrant colors.
    pub fn monokai() -> Self {
        Self {
            name: "monokai".to_string(),
            colors: [
                0x272822FF, // Black
                0xF92672FF, // Red (pink)
                0xA6E22EFF, // Green
                0xF4BF75FF, // Yellow (orange)
                0x66D9EFFF, // Blue (cyan)
                0xAE81FFFF, // Magenta (purple)
                0x66D9EFFF, // Cyan
                0xF8F8F2FF, // White
                0x75715EFF, // Bright Black
                0xFF6188FF, // Bright Red
                0xA9DC76FF, // Bright Green
                0xFFD866FF, // Bright Yellow
                0x78DCE8FF, // Bright Blue
                0xAB9DF2FF, // Bright Magenta
                0x78DCE8FF, // Bright Cyan
                0xFCFCFAFF, // Bright White
            ],
            background: 0x272822FF,
            foreground: 0xF8F8F2FF,
            cursor: 0xF8F8F0FF,
            selection: 0x49483EFF,
        }
    }

    /// Get the color for an ANSI color index.
    pub fn ansi_color(&self, color: AnsiColor) -> u32 {
        self.colors[color.as_index() as usize]
    }

    /// Check if a color matches an ANSI color in this palette.
    ///
    /// Compares the RGB channels (ignoring alpha) for an exact match.
    pub fn matches_ansi(&self, rgba: u32, color: AnsiColor) -> bool {
        let expected = self.ansi_color(color);
        // Compare RGB only (mask out alpha)
        (rgba & 0xFFFFFF00) == (expected & 0xFFFFFF00)
    }

    /// Check if a color matches an ANSI color with tolerance.
    ///
    /// Allows for slight variations in color values (useful for
    /// terminals that apply color correction or gamma adjustments).
    pub fn matches_ansi_approx(&self, rgba: u32, color: AnsiColor, tolerance: u8) -> bool {
        let expected = self.ansi_color(color);

        let r1 = ((rgba >> 24) & 0xFF) as i32;
        let g1 = ((rgba >> 16) & 0xFF) as i32;
        let b1 = ((rgba >> 8) & 0xFF) as i32;

        let r2 = ((expected >> 24) & 0xFF) as i32;
        let g2 = ((expected >> 16) & 0xFF) as i32;
        let b2 = ((expected >> 8) & 0xFF) as i32;

        let tol = tolerance as i32;
        (r1 - r2).abs() <= tol && (g1 - g2).abs() <= tol && (b1 - b2).abs() <= tol
    }
}


/// Result of scanning colors in a region.
#[derive(Debug, Clone, Default)]
pub struct ColorScan {
    /// Unique foreground colors found (RGBA format).
    pub foreground_colors: Vec<u32>,
    /// Unique background colors found (RGBA format).
    pub background_colors: Vec<u32>,
    /// Total cells scanned.
    pub cells_scanned: usize,
}

impl ColorScan {
    /// Create a new empty scan result.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a foreground color to the scan if not already present.
    pub fn add_foreground(&mut self, color: u32) {
        if !self.foreground_colors.contains(&color) {
            self.foreground_colors.push(color);
        }
    }

    /// Add a background color to the scan if not already present.
    pub fn add_background(&mut self, color: u32) {
        if !self.background_colors.contains(&color) {
            self.background_colors.push(color);
        }
    }

    /// Increment the cell counter.
    pub fn increment_cells(&mut self) {
        self.cells_scanned += 1;
    }

    /// Get the number of unique foreground colors.
    pub fn unique_foreground_count(&self) -> usize {
        self.foreground_colors.len()
    }

    /// Get the number of unique background colors.
    pub fn unique_background_count(&self) -> usize {
        self.background_colors.len()
    }
}

/// Extension trait for theme verification.
///
/// This trait provides methods for verifying colors and themes
/// in terminal output. It should be implemented by test harnesses
/// that have access to cell attributes.
pub trait ThemeTestExt {
    /// Get the cell attributes at a specific position.
    ///
    /// This is the foundational method that other methods build upon.
    fn cell_attrs_at(&self, row: u16, col: u16) -> IpcResult<CellAttributes>;

    /// Get the background color at a cell position.
    fn cell_background(&self, row: u16, col: u16) -> IpcResult<u32> {
        Ok(self.cell_attrs_at(row, col)?.bg)
    }

    /// Get the foreground color at a cell position.
    fn cell_foreground(&self, row: u16, col: u16) -> IpcResult<u32> {
        Ok(self.cell_attrs_at(row, col)?.fg)
    }

    /// Verify the terminal's current background matches expected.
    fn assert_background_color(&self, row: u16, col: u16, expected: u32) -> IpcResult<()> {
        let actual = self.cell_background(row, col)?;
        if (actual & 0xFFFFFF00) == (expected & 0xFFFFFF00) {
            Ok(())
        } else {
            Err(IpcError::InvalidData(format!(
                "Background color mismatch at ({}, {}): expected 0x{:08X}, got 0x{:08X}",
                row, col, expected, actual
            )))
        }
    }

    /// Verify the terminal's current foreground matches expected.
    fn assert_foreground_color(&self, row: u16, col: u16, expected: u32) -> IpcResult<()> {
        let actual = self.cell_foreground(row, col)?;
        if (actual & 0xFFFFFF00) == (expected & 0xFFFFFF00) {
            Ok(())
        } else {
            Err(IpcError::InvalidData(format!(
                "Foreground color mismatch at ({}, {}): expected 0x{:08X}, got 0x{:08X}",
                row, col, expected, actual
            )))
        }
    }

    /// Verify an ANSI color index maps to the expected RGB in the given palette.
    fn assert_ansi_color(
        &self,
        row: u16,
        col: u16,
        palette: &ColorPalette,
        expected_ansi: AnsiColor,
    ) -> IpcResult<()> {
        let actual = self.cell_foreground(row, col)?;
        let expected = palette.ansi_color(expected_ansi);

        if (actual & 0xFFFFFF00) == (expected & 0xFFFFFF00) {
            Ok(())
        } else {
            Err(IpcError::InvalidData(format!(
                "ANSI color {:?} mismatch at ({}, {}) using palette '{}': expected 0x{:08X}, got 0x{:08X}",
                expected_ansi, row, col, palette.name, expected, actual
            )))
        }
    }

    /// Capture the current color state at a position for snapshot testing.
    ///
    /// Returns (foreground, background) as RGBA values.
    fn capture_cell_colors(&self, row: u16, col: u16) -> IpcResult<(u32, u32)> {
        let attrs = self.cell_attrs_at(row, col)?;
        Ok((attrs.fg, attrs.bg))
    }

    /// Scan a region for all unique colors used.
    ///
    /// Scans from (start_row, start_col) to (end_row, end_col) inclusive.
    fn scan_colors_in_region(
        &self,
        start_row: u16,
        start_col: u16,
        end_row: u16,
        end_col: u16,
    ) -> IpcResult<ColorScan> {
        let mut scan = ColorScan::new();

        for row in start_row..=end_row {
            for col in start_col..=end_col {
                match self.cell_attrs_at(row, col) {
                    Ok(attrs) => {
                        scan.add_foreground(attrs.fg);
                        scan.add_background(attrs.bg);
                        scan.increment_cells();
                    }
                    Err(_) => {
                        // Skip cells that are out of bounds or invalid
                        continue;
                    }
                }
            }
        }

        Ok(scan)
    }

    /// Wait for a specific color to appear at a position.
    ///
    /// Polls the cell at (row, col) until the foreground color matches
    /// the expected value, or the timeout expires.
    fn wait_for_color(
        &self,
        row: u16,
        col: u16,
        expected_fg: u32,
        timeout: Duration,
    ) -> IpcResult<()> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(50);

        loop {
            match self.cell_foreground(row, col) {
                Ok(actual) if (actual & 0xFFFFFF00) == (expected_fg & 0xFFFFFF00) => {
                    return Ok(());
                }
                _ => {}
            }

            if start.elapsed() >= timeout {
                return Err(IpcError::Timeout(timeout));
            }

            std::thread::sleep(poll_interval);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ansi_color_indices() {
        assert_eq!(AnsiColor::Black as u8, 0);
        assert_eq!(AnsiColor::Red as u8, 1);
        assert_eq!(AnsiColor::BrightWhite as u8, 15);
    }

    #[test]
    fn test_ansi_color_from_index() {
        assert_eq!(AnsiColor::from_index(0), Some(AnsiColor::Black));
        assert_eq!(AnsiColor::from_index(1), Some(AnsiColor::Red));
        assert_eq!(AnsiColor::from_index(15), Some(AnsiColor::BrightWhite));
        assert_eq!(AnsiColor::from_index(16), None);
        assert_eq!(AnsiColor::from_index(255), None);
    }

    #[test]
    fn test_color_palette_new() {
        let palette = ColorPalette::new("test");
        assert_eq!(palette.name, "test");
        assert_eq!(palette.background, 0x000000FF);
        assert_eq!(palette.foreground, 0xFFFFFFFF);
    }

    #[test]
    fn test_slime_palette() {
        let palette = ColorPalette::slime();
        assert_eq!(palette.name, "slime");
        assert_eq!(palette.colors.len(), 16);

        // Check green color
        let green = palette.ansi_color(AnsiColor::Green);
        assert_eq!(green, 0x4ECDC4FF);
    }

    #[test]
    fn test_dracula_palette() {
        let palette = ColorPalette::dracula();
        assert_eq!(palette.name, "dracula");

        // Check signature purple
        let blue = palette.ansi_color(AnsiColor::Blue);
        assert_eq!(blue, 0xBD93F9FF);
    }

    #[test]
    fn test_nord_palette() {
        let palette = ColorPalette::nord();
        assert_eq!(palette.name, "nord");

        // Check nord blue
        let blue = palette.ansi_color(AnsiColor::Blue);
        assert_eq!(blue, 0x81A1C1FF);
    }

    #[test]
    fn test_monokai_palette() {
        let palette = ColorPalette::monokai();
        assert_eq!(palette.name, "monokai");

        // Check signature pink
        let red = palette.ansi_color(AnsiColor::Red);
        assert_eq!(red, 0xF92672FF);
    }

    #[test]
    fn test_matches_ansi() {
        let palette = ColorPalette::slime();
        let green = palette.ansi_color(AnsiColor::Green);

        assert!(palette.matches_ansi(green, AnsiColor::Green));
        assert!(!palette.matches_ansi(green, AnsiColor::Red));

        // Alpha channel should be ignored
        let green_diff_alpha = (green & 0xFFFFFF00) | 0x80;
        assert!(palette.matches_ansi(green_diff_alpha, AnsiColor::Green));
    }

    #[test]
    fn test_matches_ansi_approx() {
        let palette = ColorPalette::slime();
        let green = palette.ansi_color(AnsiColor::Green);

        // Exact match
        assert!(palette.matches_ansi_approx(green, AnsiColor::Green, 0));

        // Slightly different color within tolerance
        let r = ((green >> 24) & 0xFF) as u8;
        let g = ((green >> 16) & 0xFF) as u8;
        let b = ((green >> 8) & 0xFF) as u8;

        let similar = ((r.wrapping_add(5) as u32) << 24)
            | ((g as u32) << 16)
            | ((b as u32) << 8)
            | 0xFF;

        assert!(palette.matches_ansi_approx(similar, AnsiColor::Green, 10));
        assert!(!palette.matches_ansi_approx(similar, AnsiColor::Green, 2));
    }

    #[test]
    fn test_color_scan() {
        let mut scan = ColorScan::new();

        scan.add_foreground(0xFF0000FF);
        scan.add_foreground(0x00FF00FF);
        scan.add_foreground(0xFF0000FF); // Duplicate

        scan.add_background(0x000000FF);
        scan.add_background(0xFFFFFFFF);

        scan.increment_cells();
        scan.increment_cells();

        assert_eq!(scan.unique_foreground_count(), 2);
        assert_eq!(scan.unique_background_count(), 2);
        assert_eq!(scan.cells_scanned, 2);
    }

    #[test]
    fn test_all_palettes_have_16_colors() {
        assert_eq!(ColorPalette::slime().colors.len(), 16);
        assert_eq!(ColorPalette::dracula().colors.len(), 16);
        assert_eq!(ColorPalette::nord().colors.len(), 16);
        assert_eq!(ColorPalette::monokai().colors.len(), 16);
    }
}

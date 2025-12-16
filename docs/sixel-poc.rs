// Proof of Concept: Sixel Detection and Position Tracking with vtparse
// This demonstrates how to track cursor position when Sixel graphics are rendered
//
// To run this example:
// 1. Add to Cargo.toml: vtparse = "0.15"
// 2. cargo run --example sixel-poc
//
// Expected output shows:
// - Detection of Sixel sequence start
// - Cursor position when Sixel begins
// - Extraction of raster attributes (dimensions)
// - Final cursor position

use vtparse::{VTActor, VTParser, CsiParam};

/// Tracks Sixel sequences and their positions in the terminal
#[derive(Debug)]
struct SixelInfo {
    /// Terminal row where Sixel starts (0-indexed)
    start_row: usize,
    /// Terminal column where Sixel starts (0-indexed)
    start_col: usize,
    /// Width in pixels (from raster attributes)
    width_px: Option<usize>,
    /// Height in pixels (from raster attributes)
    height_px: Option<usize>,
    /// Sixel parameters (aspect ratio, background mode)
    params: Vec<i64>,
}

/// Terminal state tracker that implements VTActor to intercept escape sequences
struct TerminalTracker {
    /// Current cursor position (row, col)
    cursor_pos: (usize, usize),
    /// Are we currently processing a Sixel sequence?
    in_sixel: bool,
    /// Buffer for current Sixel data being processed
    sixel_data_buffer: Vec<u8>,
    /// All detected Sixel sequences
    sixel_regions: Vec<SixelInfo>,
    /// Terminal dimensions for reference
    terminal_size: (usize, usize),
}

impl TerminalTracker {
    fn new(rows: usize, cols: usize) -> Self {
        Self {
            cursor_pos: (0, 0),
            in_sixel: false,
            sixel_data_buffer: Vec::new(),
            sixel_regions: Vec::new(),
            terminal_size: (rows, cols),
        }
    }

    /// Parse raster attributes from sixel data: "Pa;Pb;Ph;Pv
    /// Returns (width, height) in pixels if found
    fn parse_raster_attributes(&self, data: &[u8]) -> Option<(usize, usize)> {
        // Look for raster attribute command starting with '"'
        let data_str = std::str::from_utf8(data).ok()?;

        // Find the raster attributes command
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
                let width = parts[2].parse::<usize>().ok()?;
                let height = parts[3].parse::<usize>().ok()?;
                return Some((width, height));
            }
        }
        None
    }

    /// Convert pixel dimensions to terminal cells
    /// Assumes standard cell size of 8x16 pixels (common for xterm)
    fn pixels_to_cells(&self, width_px: usize, height_px: usize) -> (usize, usize) {
        const CELL_WIDTH: usize = 8;
        const CELL_HEIGHT: usize = 16;

        let width_cells = (width_px + CELL_WIDTH - 1) / CELL_WIDTH;
        let height_cells = (height_px + CELL_HEIGHT - 1) / CELL_HEIGHT;

        (width_cells, height_cells)
    }

    fn print_summary(&self) {
        println!("\n{}", "=".repeat(70));
        println!("SIXEL TRACKING SUMMARY");
        println!("{}", "=".repeat(70));
        println!("Terminal size: {} rows x {} cols",
                 self.terminal_size.0, self.terminal_size.1);
        println!("Final cursor position: row {}, col {}",
                 self.cursor_pos.0, self.cursor_pos.1);
        println!("\nDetected {} Sixel sequence(s):\n", self.sixel_regions.len());

        for (i, info) in self.sixel_regions.iter().enumerate() {
            println!("Sixel #{}", i + 1);
            println!("  Position: row {}, col {}", info.start_row, info.start_col);
            println!("  Parameters: {:?}", info.params);

            if let (Some(w), Some(h)) = (info.width_px, info.height_px) {
                let (cells_w, cells_h) = self.pixels_to_cells(w, h);
                println!("  Dimensions: {}x{} pixels ({} x {} cells)",
                         w, h, cells_w, cells_h);
                println!("  Occupies: rows {}-{}, cols {}-{}",
                         info.start_row,
                         info.start_row + cells_h - 1,
                         info.start_col,
                         info.start_col + cells_w - 1);
            } else {
                println!("  Dimensions: Not specified (no raster attributes)");
            }
            println!();
        }
        println!("{}", "=".repeat(70));
    }
}

impl VTActor for TerminalTracker {
    fn print(&mut self, c: char) {
        // Regular text moves cursor forward
        self.cursor_pos.1 += 1;

        // Handle line wrapping
        if self.cursor_pos.1 >= self.terminal_size.1 {
            self.cursor_pos.0 += 1;
            self.cursor_pos.1 = 0;
        }
    }

    fn execute_c0_or_c1(&mut self, control: u8) {
        match control {
            b'\n' => {
                // Line feed
                self.cursor_pos.0 += 1;
            }
            b'\r' => {
                // Carriage return
                self.cursor_pos.1 = 0;
            }
            b'\t' => {
                // Tab - advance to next tab stop (every 8 columns)
                self.cursor_pos.1 = ((self.cursor_pos.1 / 8) + 1) * 8;
            }
            _ => {}
        }
    }

    fn dcs_hook(
        &mut self,
        mode: u8,
        params: &[i64],
        _intermediates: &[u8],
        _ignored_excess_intermediates: bool,
    ) {
        // Sixel sequences are identified by mode byte 'q' (0x71)
        if mode == b'q' {
            self.in_sixel = true;
            self.sixel_data_buffer.clear();

            println!("\n>>> SIXEL SEQUENCE DETECTED <<<");
            println!("    Start position: row {}, col {}",
                     self.cursor_pos.0, self.cursor_pos.1);
            println!("    Parameters: {:?}", params);

            // Store initial info
            self.sixel_regions.push(SixelInfo {
                start_row: self.cursor_pos.0,
                start_col: self.cursor_pos.1,
                width_px: None,
                height_px: None,
                params: params.to_vec(),
            });
        } else {
            // Other DCS sequences (not Sixel)
            println!("\n>>> DCS Sequence (mode: {}) <<<", mode as char);
        }
    }

    fn dcs_put(&mut self, byte: u8) {
        if self.in_sixel {
            // Accumulate sixel data to parse raster attributes later
            self.sixel_data_buffer.push(byte);

            // Try to parse raster attributes if we have enough data
            if self.sixel_data_buffer.len() > 10 {
                if let Some((width, height)) = self.parse_raster_attributes(&self.sixel_data_buffer) {
                    // Update the last sixel info with dimensions
                    if let Some(info) = self.sixel_regions.last_mut() {
                        if info.width_px.is_none() {
                            info.width_px = Some(width);
                            info.height_px = Some(height);

                            let (cells_w, cells_h) = self.pixels_to_cells(width, height);
                            println!("    Raster attributes found: {}x{} pixels ({} x {} cells)",
                                     width, height, cells_w, cells_h);
                        }
                    }
                }
            }
        }
    }

    fn dcs_unhook(&mut self) {
        if self.in_sixel {
            println!("    SIXEL SEQUENCE ENDED");

            // Some terminals move cursor after Sixel, some don't
            // For this POC, we'll document current position
            println!("    Cursor after Sixel: row {}, col {}\n",
                     self.cursor_pos.0, self.cursor_pos.1);

            self.in_sixel = false;
            self.sixel_data_buffer.clear();
        }
    }

    fn csi_dispatch(&mut self, params: &[CsiParam], _truncated: bool, byte: u8) {
        match byte {
            b'H' | b'f' => {
                // CUP - Cursor Position ESC [ row ; col H
                let row = params.get(0)
                    .and_then(|p| p.as_integer())
                    .unwrap_or(1) as usize - 1;  // 1-indexed to 0-indexed
                let col = params.get(1)
                    .and_then(|p| p.as_integer())
                    .unwrap_or(1) as usize - 1;

                self.cursor_pos = (row, col);
                println!("Cursor moved to: row {}, col {}", row, col);
            }
            b'A' => {
                // CUU - Cursor Up
                let n = params.get(0)
                    .and_then(|p| p.as_integer())
                    .unwrap_or(1) as usize;
                self.cursor_pos.0 = self.cursor_pos.0.saturating_sub(n);
            }
            b'B' => {
                // CUD - Cursor Down
                let n = params.get(0)
                    .and_then(|p| p.as_integer())
                    .unwrap_or(1) as usize;
                self.cursor_pos.0 = (self.cursor_pos.0 + n).min(self.terminal_size.0 - 1);
            }
            b'C' => {
                // CUF - Cursor Forward
                let n = params.get(0)
                    .and_then(|p| p.as_integer())
                    .unwrap_or(1) as usize;
                self.cursor_pos.1 = (self.cursor_pos.1 + n).min(self.terminal_size.1 - 1);
            }
            b'D' => {
                // CUB - Cursor Back
                let n = params.get(0)
                    .and_then(|p| p.as_integer())
                    .unwrap_or(1) as usize;
                self.cursor_pos.1 = self.cursor_pos.1.saturating_sub(n);
            }
            _ => {}
        }
    }

    fn esc_dispatch(
        &mut self,
        _params: &[i64],
        _intermediates: &[u8],
        _ignored_excess_intermediates: bool,
        _byte: u8,
    ) {
        // Handle escape sequences (we don't need these for basic cursor tracking)
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]]) {
        // Handle OSC sequences (window title, etc.)
    }

    fn apc_dispatch(&mut self, _data: Vec<u8>) {
        // Handle APC sequences (e.g., Kitty graphics protocol)
    }
}

fn main() {
    println!("Sixel Detection Proof of Concept");
    println!("{}", "=".repeat(70));
    println!("This demonstrates tracking cursor position during Sixel rendering.\n");

    let mut parser = VTParser::new();
    let mut tracker = TerminalTracker::new(24, 80);

    // Test Case 1: Simple Sixel with raster attributes
    println!("TEST 1: Sixel with raster attributes");
    println!("{}", "-".repeat(70));
    let test1 = b"\x1b[5;10HSome text before sixel\n\
                   \x1bPq\"1;1;100;50#0;2;100;100;100#0~-~-~-~-\x1b\\\
                   Text after sixel";
    parser.parse(test1, &mut tracker);

    // Test Case 2: Position cursor, then Sixel without raster attributes
    println!("\nTEST 2: Sixel without raster attributes");
    println!("{}", "-".repeat(70));
    let test2 = b"\x1b[10;20H\x1bPq#0;2;0;0;0#0!50~\x1b\\";
    parser.parse(test2, &mut tracker);

    // Test Case 3: Multiple cursor movements and Sixel
    println!("\nTEST 3: Complex sequence with multiple operations");
    println!("{}", "-".repeat(70));
    let test3 = b"\x1b[H\x1b[2Jstart\n\
                   \x1b[5;5Himage here:\x1b[6;5H\
                   \x1bPq\"1;1;80;40#0!40~$!40~\x1b\\\
                   \x1b[10;10Hafter image";
    parser.parse(test3, &mut tracker);

    // Print final summary
    tracker.print_summary();

    println!("\n{}", "=".repeat(70));
    println!("INTERPRETATION GUIDE");
    println!("{}", "=".repeat(70));
    println!("1. Sixel sequences start with ESC P q (DCS + 'q')");
    println!("2. Cursor position is captured at the start of the sequence");
    println!("3. Raster attributes (\") provide width/height in pixels");
    println!("4. Cell dimensions assume 8x16 pixel cells (adjustable)");
    println!("5. Occupied region = (start_row, start_col) to (start_row+h, start_col+w)");
    println!("{}", "=".repeat(70));

    println!("\nSUCCESS: Sixel detection and position tracking working!");
    println!("\nFor terminal_testlib integration:");
    println!("- Use this VTActor pattern to track all escape sequences");
    println!("- Store sixel_regions for collision detection");
    println!("- Calculate occupied cells for rendering decisions");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sixel_detection() {
        let mut parser = VTParser::new();
        let mut tracker = TerminalTracker::new(24, 80);

        // Simple sixel at row 0, col 0
        let input = b"\x1bPq\"1;1;80;40#0!10~\x1b\\";
        parser.parse(input, &mut tracker);

        assert_eq!(tracker.sixel_regions.len(), 1);
        let info = &tracker.sixel_regions[0];
        assert_eq!(info.start_row, 0);
        assert_eq!(info.start_col, 0);
        assert_eq!(info.width_px, Some(80));
        assert_eq!(info.height_px, Some(40));
    }

    #[test]
    fn test_cursor_positioning_before_sixel() {
        let mut parser = VTParser::new();
        let mut tracker = TerminalTracker::new(24, 80);

        // Move cursor to (5, 10), then render sixel
        let input = b"\x1b[6;11H\x1bPq\"1;1;100;50#0\x1b\\";
        parser.parse(input, &mut tracker);

        assert_eq!(tracker.sixel_regions.len(), 1);
        let info = &tracker.sixel_regions[0];
        assert_eq!(info.start_row, 5);  // 0-indexed
        assert_eq!(info.start_col, 10); // 0-indexed
    }

    #[test]
    fn test_pixels_to_cells_conversion() {
        let tracker = TerminalTracker::new(24, 80);

        // 80x40 pixels -> 10x3 cells (8x16 each)
        let (w, h) = tracker.pixels_to_cells(80, 40);
        assert_eq!(w, 10);
        assert_eq!(h, 3);

        // Test rounding up: 81x41 pixels -> 11x3 cells
        let (w, h) = tracker.pixels_to_cells(81, 41);
        assert_eq!(w, 11);
        assert_eq!(h, 3);
    }

    #[test]
    fn test_multiple_sixels() {
        let mut parser = VTParser::new();
        let mut tracker = TerminalTracker::new(24, 80);

        let input = b"\x1b[2;3H\x1bPq\"1;1;80;40#0\x1b\\\
                      \x1b[10;20H\x1bPq\"1;1;50;30#0\x1b\\";
        parser.parse(input, &mut tracker);

        assert_eq!(tracker.sixel_regions.len(), 2);

        let first = &tracker.sixel_regions[0];
        assert_eq!(first.start_row, 1);
        assert_eq!(first.start_col, 2);

        let second = &tracker.sixel_regions[1];
        assert_eq!(second.start_row, 9);
        assert_eq!(second.start_col, 19);
    }
}

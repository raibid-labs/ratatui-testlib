//! Terminal screen state management using vtparse with Sixel support.
//!
//! This module provides the core terminal emulation layer that tracks screen contents,
//! cursor position, and Sixel graphics regions. It uses the [`vtparse`] crate to parse
//! VT100/ANSI escape sequences.
//!
//! # Key Types
//!
//! - [`ScreenState`]: The main screen state tracking type
//! - [`SixelRegion`]: Represents a Sixel graphics region with position and dimension info
//!
//! # Example
//!
//! ```rust
//! use term_test::ScreenState;
//!
//! let mut screen = ScreenState::new(80, 24);
//!
//! // Feed terminal output
//! screen.feed(b"Hello, World!");
//!
//! // Query screen contents
//! assert!(screen.contains("Hello"));
//! assert_eq!(screen.cursor_position(), (0, 13));
//!
//! // Check specific position
//! assert_eq!(screen.text_at(0, 0), Some('H'));
//! ```

use vtparse::{VTActor, VTParser, CsiParam};

/// Represents a Sixel graphics region in the terminal.
///
/// Sixel is a bitmap graphics format used by terminals to display images.
/// This struct tracks the position and dimensions of Sixel graphics rendered
/// on the screen, which is essential for verifying that graphics appear in
/// the correct locations (e.g., within preview areas).
///
/// # Fields
///
/// - `start_row`: The row where the Sixel begins (0-indexed)
/// - `start_col`: The column where the Sixel begins (0-indexed)
/// - `width`: Width of the Sixel image in pixels
/// - `height`: Height of the Sixel image in pixels
/// - `data`: The raw Sixel escape sequence data
///
/// # Example
///
/// ```rust
/// # use term_test::ScreenState;
/// let mut screen = ScreenState::new(80, 24);
///
/// // After rendering a Sixel image...
/// let regions = screen.sixel_regions();
/// for region in regions {
///     println!("Sixel at ({}, {}), size {}x{}",
///         region.start_row, region.start_col,
///         region.width, region.height);
/// }
/// ```
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
    /// Raw Sixel escape sequence data.
    pub data: Vec<u8>,
}

/// Terminal state tracking for vtparse parser.
///
/// Implements VTActor to handle escape sequences including DCS for Sixel.
struct TerminalState {
    cursor_pos: (u16, u16),
    sixel_regions: Vec<SixelRegion>,
    current_sixel_data: Vec<u8>,
    current_sixel_params: Vec<i64>,
    in_sixel_mode: bool,
    width: u16,
    height: u16,
    cells: Vec<Vec<char>>,
}

impl TerminalState {
    fn new(width: u16, height: u16) -> Self {
        let cells = vec![vec![' '; width as usize]; height as usize];

        Self {
            cursor_pos: (0, 0),
            sixel_regions: Vec::new(),
            current_sixel_data: Vec::new(),
            current_sixel_params: Vec::new(),
            in_sixel_mode: false,
            width,
            height,
            cells,
        }
    }

    fn put_char(&mut self, ch: char) {
        let (row, col) = self.cursor_pos;
        if row < self.height && col < self.width {
            self.cells[row as usize][col as usize] = ch;
            // Move cursor forward, but don't wrap automatically
            if col + 1 < self.width {
                self.cursor_pos.1 = col + 1;
            }
        }
    }

    fn move_cursor(&mut self, row: u16, col: u16) {
        self.cursor_pos = (row.min(self.height - 1), col.min(self.width - 1));
    }

    /// Parse raster attributes from sixel data: "Pa;Pb;Ph;Pv
    /// Returns (width, height) in pixels if found
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
}

impl VTActor for TerminalState {
    fn print(&mut self, ch: char) {
        self.put_char(ch);
    }

    fn execute_c0_or_c1(&mut self, control: u8) {
        match control {
            b'\r' => {
                // Carriage return
                self.cursor_pos.1 = 0;
            }
            b'\n' => {
                // Line feed
                if self.cursor_pos.0 + 1 < self.height {
                    self.cursor_pos.0 += 1;
                }
            }
            b'\t' => {
                // Tab - advance to next tab stop (every 8 columns)
                let next_tab = ((self.cursor_pos.1 / 8) + 1) * 8;
                self.cursor_pos.1 = next_tab.min(self.width - 1);
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
            self.in_sixel_mode = true;
            self.current_sixel_data.clear();
            self.current_sixel_params = params.to_vec();
        }
    }

    fn dcs_put(&mut self, byte: u8) {
        if self.in_sixel_mode {
            self.current_sixel_data.push(byte);
        }
    }

    fn dcs_unhook(&mut self) {
        if self.in_sixel_mode {
            // Parse dimensions from raster attributes if present
            let (width, height) = self
                .parse_raster_attributes(&self.current_sixel_data)
                .unwrap_or((0, 0));

            let region = SixelRegion {
                start_row: self.cursor_pos.0,
                start_col: self.cursor_pos.1,
                width,
                height,
                data: self.current_sixel_data.clone(),
            };
            self.sixel_regions.push(region);

            self.in_sixel_mode = false;
            self.current_sixel_data.clear();
            self.current_sixel_params.clear();
        }
    }

    fn csi_dispatch(&mut self, params: &[CsiParam], _truncated: bool, byte: u8) {
        match byte {
            b'H' | b'f' => {
                // CUP - Cursor Position ESC [ row ; col H
                // CSI uses 1-based indexing, convert to 0-based
                // Filter out P variants (separators) and collect only integers
                let integers: Vec<i64> = params
                    .iter()
                    .filter_map(|p| p.as_integer())
                    .collect();

                let row = integers
                    .get(0)
                    .copied()
                    .unwrap_or(1)
                    .saturating_sub(1) as u16;
                let col = integers
                    .get(1)
                    .copied()
                    .unwrap_or(1)
                    .saturating_sub(1) as u16;

                self.move_cursor(row, col);
            }
            b'A' => {
                // CUU - Cursor Up
                let n = params
                    .iter()
                    .find_map(|p| p.as_integer())
                    .unwrap_or(1) as u16;
                self.cursor_pos.0 = self.cursor_pos.0.saturating_sub(n);
            }
            b'B' => {
                // CUD - Cursor Down
                let n = params
                    .iter()
                    .find_map(|p| p.as_integer())
                    .unwrap_or(1) as u16;
                self.cursor_pos.0 = (self.cursor_pos.0 + n).min(self.height - 1);
            }
            b'C' => {
                // CUF - Cursor Forward
                let n = params
                    .iter()
                    .find_map(|p| p.as_integer())
                    .unwrap_or(1) as u16;
                self.cursor_pos.1 = (self.cursor_pos.1 + n).min(self.width - 1);
            }
            b'D' => {
                // CUB - Cursor Back
                let n = params
                    .iter()
                    .find_map(|p| p.as_integer())
                    .unwrap_or(1) as u16;
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
        byte: u8,
    ) {
        match byte {
            b'D' => {
                // IND - Index (move cursor down)
                if self.cursor_pos.0 + 1 < self.height {
                    self.cursor_pos.0 += 1;
                }
            }
            b'E' => {
                // NEL - Next Line
                if self.cursor_pos.0 + 1 < self.height {
                    self.cursor_pos.0 += 1;
                }
                self.cursor_pos.1 = 0;
            }
            _ => {}
        }
    }

    fn osc_dispatch(&mut self, _params: &[&[u8]]) {
        // Handle OSC sequences (window title, etc.)
        // Not needed for basic functionality
    }

    fn apc_dispatch(&mut self, _data: Vec<u8>) {
        // Handle APC sequences (e.g., Kitty graphics protocol)
        // Not needed for basic functionality
    }
}

/// Represents the current state of the terminal screen.
///
/// `ScreenState` is the core terminal emulator that tracks:
/// - Text content at each cell position
/// - Current cursor position
/// - Sixel graphics regions (when rendered via DCS sequences)
///
/// It wraps a [`vtparse`] parser that processes VT100/ANSI escape sequences
/// and maintains the screen state accordingly.
///
/// # Usage
///
/// The typical workflow is:
/// 1. Create a `ScreenState` with desired dimensions
/// 2. Feed PTY output bytes using [`feed()`](Self::feed)
/// 3. Query the state using various accessor methods
///
/// # Example
///
/// ```rust
/// use term_test::ScreenState;
///
/// let mut screen = ScreenState::new(80, 24);
///
/// // Feed some terminal output
/// screen.feed(b"\x1b[2J"); // Clear screen
/// screen.feed(b"\x1b[5;10H"); // Move cursor to (5, 10)
/// screen.feed(b"Hello!");
///
/// // Query the state
/// assert_eq!(screen.cursor_position(), (4, 16)); // 0-indexed
/// assert_eq!(screen.text_at(4, 9), Some('H'));
/// assert!(screen.contains("Hello"));
/// ```
pub struct ScreenState {
    parser: VTParser,
    state: TerminalState,
    width: u16,
    height: u16,
}

impl ScreenState {
    /// Creates a new screen state with the specified dimensions.
    ///
    /// Initializes an empty screen filled with spaces, with the cursor at (0, 0).
    ///
    /// # Arguments
    ///
    /// * `width` - Screen width in columns
    /// * `height` - Screen height in rows
    ///
    /// # Example
    ///
    /// ```rust
    /// use term_test::ScreenState;
    ///
    /// let screen = ScreenState::new(80, 24);
    /// assert_eq!(screen.size(), (80, 24));
    /// assert_eq!(screen.cursor_position(), (0, 0));
    /// ```
    pub fn new(width: u16, height: u16) -> Self {
        let parser = VTParser::new();
        let state = TerminalState::new(width, height);

        Self {
            parser,
            state,
            width,
            height,
        }
    }

    /// Feeds data from the PTY to the parser.
    ///
    /// This processes VT100/ANSI escape sequences and updates the screen state,
    /// including:
    /// - Text output
    /// - Cursor movements
    /// - Sixel graphics (tracked via DCS callbacks)
    ///
    /// This method can be called multiple times to incrementally feed data.
    /// The parser maintains state across calls, so partial escape sequences
    /// are handled correctly.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw bytes from PTY output
    ///
    /// # Example
    ///
    /// ```rust
    /// use term_test::ScreenState;
    ///
    /// let mut screen = ScreenState::new(80, 24);
    ///
    /// // Feed data incrementally
    /// screen.feed(b"Hello, ");
    /// screen.feed(b"World!");
    ///
    /// assert!(screen.contains("Hello, World!"));
    /// ```
    pub fn feed(&mut self, data: &[u8]) {
        self.parser.parse(data, &mut self.state);
    }

    /// Returns the screen contents as a string.
    ///
    /// This includes all visible characters, preserving layout with newlines
    /// between rows. Empty cells are represented as spaces.
    ///
    /// # Returns
    ///
    /// A string containing the entire screen contents, with rows separated by newlines.
    ///
    /// # Example
    ///
    /// ```rust
    /// use term_test::ScreenState;
    ///
    /// let mut screen = ScreenState::new(10, 3);
    /// screen.feed(b"Hello");
    ///
    /// let contents = screen.contents();
    /// // First line contains "Hello     " (padded to 10 chars)
    /// // Second and third lines are all spaces
    /// assert!(contents.contains("Hello"));
    /// ```
    pub fn contents(&self) -> String {
        self.state
            .cells
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Returns the contents of a specific row.
    ///
    /// # Arguments
    ///
    /// * `row` - Row index (0-based)
    ///
    /// # Returns
    ///
    /// The row contents as a string, or empty string if row is out of bounds.
    pub fn row_contents(&self, row: u16) -> String {
        if row < self.height {
            self.state.cells[row as usize].iter().collect()
        } else {
            String::new()
        }
    }

    /// Returns the character at a specific position.
    ///
    /// # Arguments
    ///
    /// * `row` - Row index (0-based)
    /// * `col` - Column index (0-based)
    ///
    /// # Returns
    ///
    /// The character at the position, or None if out of bounds.
    pub fn text_at(&self, row: u16, col: u16) -> Option<char> {
        if row < self.height && col < self.width {
            Some(self.state.cells[row as usize][col as usize])
        } else {
            None
        }
    }

    /// Returns the current cursor position.
    ///
    /// # Returns
    ///
    /// A tuple of (row, col) with 0-based indexing.
    pub fn cursor_position(&self) -> (u16, u16) {
        self.state.cursor_pos
    }

    /// Returns the screen dimensions.
    ///
    /// # Returns
    ///
    /// A tuple of (width, height) in columns and rows.
    pub fn size(&self) -> (u16, u16) {
        (self.width, self.height)
    }

    /// Returns all Sixel graphics regions currently on screen.
    ///
    /// This method provides access to all Sixel graphics that have been rendered
    /// via DCS (Device Control String) sequences. Each region includes position
    /// and dimension information.
    ///
    /// This is essential for verifying Sixel positioning in tests, particularly
    /// for ensuring that graphics appear within designated preview areas.
    ///
    /// # Returns
    ///
    /// A slice of [`SixelRegion`] containing all detected Sixel graphics.
    ///
    /// # Example
    ///
    /// ```rust
    /// use term_test::ScreenState;
    ///
    /// let mut screen = ScreenState::new(80, 24);
    /// // ... render some Sixel graphics ...
    ///
    /// let regions = screen.sixel_regions();
    /// for (i, region) in regions.iter().enumerate() {
    ///     println!("Region {}: position ({}, {}), size {}x{}",
    ///         i, region.start_row, region.start_col,
    ///         region.width, region.height);
    /// }
    /// ```
    pub fn sixel_regions(&self) -> &[SixelRegion] {
        &self.state.sixel_regions
    }

    /// Checks if a Sixel region exists at the given position.
    ///
    /// This method checks if any Sixel region has its starting position
    /// at the exact (row, col) coordinates provided.
    ///
    /// # Arguments
    ///
    /// * `row` - Row to check (0-indexed)
    /// * `col` - Column to check (0-indexed)
    ///
    /// # Returns
    ///
    /// `true` if a Sixel region starts at the given position, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use term_test::ScreenState;
    ///
    /// let mut screen = ScreenState::new(80, 24);
    /// // ... render Sixel at position (5, 10) ...
    ///
    /// assert!(screen.has_sixel_at(5, 10));
    /// assert!(!screen.has_sixel_at(0, 0));
    /// ```
    pub fn has_sixel_at(&self, row: u16, col: u16) -> bool {
        self.state.sixel_regions.iter().any(|region| {
            region.start_row == row && region.start_col == col
        })
    }

    /// Returns the screen contents for debugging purposes.
    ///
    /// This is currently an alias for [`contents()`](Self::contents), but may
    /// include additional debug information in the future.
    ///
    /// # Returns
    ///
    /// A string containing the screen contents.
    pub fn debug_contents(&self) -> String {
        self.contents()
    }

    /// Checks if the screen contains the specified text.
    ///
    /// This is a convenience method that searches the entire screen contents
    /// for the given substring. It's useful for simple text-based assertions
    /// in tests.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to search for
    ///
    /// # Returns
    ///
    /// `true` if the text appears anywhere on the screen, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust
    /// use term_test::ScreenState;
    ///
    /// let mut screen = ScreenState::new(80, 24);
    /// screen.feed(b"Welcome to the application");
    ///
    /// assert!(screen.contains("Welcome"));
    /// assert!(screen.contains("application"));
    /// assert!(!screen.contains("goodbye"));
    /// ```
    pub fn contains(&self, text: &str) -> bool {
        self.contents().contains(text)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_screen() {
        let screen = ScreenState::new(80, 24);
        assert_eq!(screen.size(), (80, 24));
    }

    #[test]
    fn test_feed_simple_text() {
        let mut screen = ScreenState::new(80, 24);
        screen.feed(b"Hello, World!");
        assert!(screen.contents().contains("Hello, World!"));
    }

    #[test]
    fn test_cursor_position() {
        let mut screen = ScreenState::new(80, 24);

        // Initial position
        assert_eq!(screen.cursor_position(), (0, 0));

        // Move cursor using CSI sequence (ESC [ 5 ; 10 H = row 5, col 10)
        screen.feed(b"\x1b[5;10H");
        let (row, col) = screen.cursor_position();

        // CSI uses 1-based, we convert to 0-based
        assert_eq!(row, 4);  // 5-1 = 4
        assert_eq!(col, 9);  // 10-1 = 9
    }

    #[test]
    fn test_text_at() {
        let mut screen = ScreenState::new(80, 24);
        screen.feed(b"Test");

        assert_eq!(screen.text_at(0, 0), Some('T'));
        assert_eq!(screen.text_at(0, 1), Some('e'));
        assert_eq!(screen.text_at(0, 2), Some('s'));
        assert_eq!(screen.text_at(0, 3), Some('t'));
        assert_eq!(screen.text_at(0, 4), Some(' '));
        assert_eq!(screen.text_at(100, 100), None);
    }
}

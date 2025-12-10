//! OSC 133 semantic zone testing helpers.
//!
//! This module provides testing utilities for OSC 133 semantic zones, which are
//! part of the shell integration protocol that marks different phases of command
//! execution in the terminal.
//!
//! # OSC 133 Markers
//!
//! - **A**: Fresh line (start of prompt)
//! - **B**: Start of command input
//! - **C**: End of command (execution starting)
//! - **D**: End of output (with optional exit code)
//!
//! # Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "ipc")]
//! # {
//! use std::time::Duration;
//! use ratatui_testlib::{
//!     zones::{SemanticZoneExt, ZoneType},
//!     scarab::ScarabTestHarness,
//! };
//!
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! let mut harness = ScarabTestHarness::connect()?;
//!
//! // Send a command
//! harness.send_input("echo hello\n")?;
//!
//! // Wait for command to complete
//! let exit_code = harness.wait_for_command_complete(Duration::from_secs(5))?;
//! assert_eq!(exit_code, Some(0));
//!
//! // Get the last output zone
//! let output = harness.last_output_zone()?;
//! if let Some(zone) = output {
//!     let text = harness.zone_text(&zone)?;
//!     assert!(text.contains("hello"));
//! }
//! # Ok(())
//! # }
//! # }
//! ```

use std::time::Duration;

use crate::ipc::IpcResult;

/// Represents a semantic zone from OSC 133.
///
/// A semantic zone is a rectangular region in the terminal output that
/// corresponds to a specific phase of command execution (prompt, command, or output).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SemanticZone {
    /// Type of zone.
    pub zone_type: ZoneType,
    /// Starting row (0-indexed).
    pub start_row: u16,
    /// Starting column (0-indexed).
    pub start_col: u16,
    /// Ending row (0-indexed).
    pub end_row: u16,
    /// Ending column (0-indexed).
    pub end_col: u16,
    /// Exit code (only for Output zones after D marker).
    pub exit_code: Option<i32>,
}

/// Type of semantic zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoneType {
    /// Prompt area (from A to B).
    Prompt,
    /// Command text (from B to C).
    Command,
    /// Command output (from C to D).
    Output,
}

/// Marker type for OSC 133 sequences.
///
/// Each marker represents a transition point in the command execution lifecycle.
///
/// # OSC 133 Sequence Format
///
/// - `\x1b]133;A\x07` - Fresh line/start of prompt
/// - `\x1b]133;B\x07` - Start of command
/// - `\x1b]133;C\x07` - End of command/start of output
/// - `\x1b]133;D;0\x07` - End of output with exit code 0
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Osc133Marker {
    /// A - Fresh line/start of prompt
    FreshLine,
    /// B - Start of command
    CommandStart,
    /// C - End of command/start of output
    CommandExecuted,
    /// D - End of output (with optional exit code)
    CommandFinished(Option<i32>),
}

impl Osc133Marker {
    /// Parse an OSC 133 marker from the parameters string.
    ///
    /// # Arguments
    ///
    /// * `params` - The parameters after "133;" (e.g., "A", "D;0")
    fn from_params(params: &str) -> Option<Self> {
        let parts: Vec<&str> = params.split(';').collect();
        match parts.first()? {
            &"A" => Some(Osc133Marker::FreshLine),
            &"B" => Some(Osc133Marker::CommandStart),
            &"C" => Some(Osc133Marker::CommandExecuted),
            &"D" => {
                let exit_code = if parts.len() > 1 {
                    parts[1].parse::<i32>().ok()
                } else {
                    None
                };
                Some(Osc133Marker::CommandFinished(exit_code))
            }
            _ => None,
        }
    }
}

/// Parser for OSC 133 sequences in terminal output.
///
/// This parser tracks OSC 133 markers encountered during terminal parsing
/// and constructs semantic zones from consecutive markers.
///
/// # Example
///
/// ```rust
/// use ratatui_testlib::zones::Osc133Parser;
///
/// let mut parser = Osc133Parser::new();
///
/// // Parse OSC 133 sequences from terminal data
/// let data = b"\x1b]133;A\x07$ \x1b]133;B\x07ls\x1b]133;C\x07\nfile.txt\n\x1b]133;D;0\x07";
/// parser.parse(data);
///
/// let zones = parser.zones();
/// assert_eq!(zones.len(), 3); // Prompt, Command, Output
/// ```
#[derive(Debug)]
pub struct Osc133Parser {
    markers: Vec<(Osc133Marker, u16, u16)>, // (marker, row, col)
}

impl Osc133Parser {
    /// Create a new OSC 133 parser.
    pub fn new() -> Self {
        Self {
            markers: Vec::new(),
        }
    }

    /// Parse OSC 133 markers from raw terminal data.
    ///
    /// This scans through the data looking for OSC 133 sequences and records
    /// their positions. The position tracking assumes the data represents a
    /// sequential stream of terminal output.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw terminal data that may contain OSC 133 sequences
    pub fn parse(&mut self, data: &[u8]) {
        let mut row: u16 = 0;
        let mut col: u16 = 0;
        let mut i = 0;

        while i < data.len() {
            // Look for OSC start: ESC ]
            if i + 1 < data.len() && data[i] == 0x1b && data[i + 1] == b']' {
                i += 2;

                // Look for "133;"
                if i + 4 < data.len()
                    && data[i] == b'1'
                    && data[i + 1] == b'3'
                    && data[i + 2] == b'3'
                    && data[i + 3] == b';'
                {
                    i += 4;

                    // Find the terminator (BEL or ST)
                    let mut end = i;
                    while end < data.len() {
                        if data[end] == 0x07 {
                            // BEL
                            break;
                        }
                        if end + 1 < data.len() && data[end] == 0x1b && data[end + 1] == b'\\' {
                            // ST
                            break;
                        }
                        end += 1;
                    }

                    // Parse the parameters
                    if end > i {
                        if let Ok(params) = std::str::from_utf8(&data[i..end]) {
                            if let Some(marker) = Osc133Marker::from_params(params) {
                                self.markers.push((marker, row, col));
                            }
                        }
                    }

                    i = end + 1;
                    continue;
                }
            }

            // Track position for visible characters
            match data[i] {
                b'\n' => {
                    row += 1;
                    col = 0;
                }
                b'\r' => {
                    col = 0;
                }
                0x1b => {
                    // Skip other escape sequences
                    i += 1;
                    if i < data.len() && data[i] == b'[' {
                        // CSI sequence
                        i += 1;
                        while i < data.len() && data[i] >= 0x20 && data[i] < 0x40 {
                            i += 1;
                        }
                    }
                }
                _ if data[i] >= 0x20 => {
                    col += 1;
                }
                _ => {}
            }

            i += 1;
        }
    }

    /// Get all detected zones.
    ///
    /// Constructs semantic zones from consecutive markers. A zone is created
    /// between two consecutive markers:
    /// - Prompt zone: from A to B
    /// - Command zone: from B to C
    /// - Output zone: from C to D
    pub fn zones(&self) -> Vec<SemanticZone> {
        let mut zones = Vec::new();

        for i in 0..self.markers.len().saturating_sub(1) {
            let (marker, start_row, start_col) = self.markers[i];
            let (next_marker, end_row, end_col) = self.markers[i + 1];

            let zone_type = match (marker, next_marker) {
                (Osc133Marker::FreshLine, Osc133Marker::CommandStart) => Some(ZoneType::Prompt),
                (Osc133Marker::CommandStart, Osc133Marker::CommandExecuted) => {
                    Some(ZoneType::Command)
                }
                (Osc133Marker::CommandExecuted, Osc133Marker::CommandFinished(exit_code)) => {
                    zones.push(SemanticZone {
                        zone_type: ZoneType::Output,
                        start_row,
                        start_col,
                        end_row,
                        end_col,
                        exit_code,
                    });
                    None
                }
                _ => None,
            };

            if let Some(zone_type) = zone_type {
                zones.push(SemanticZone {
                    zone_type,
                    start_row,
                    start_col,
                    end_row,
                    end_col,
                    exit_code: None,
                });
            }
        }

        zones
    }

    /// Clear all parsed markers.
    ///
    /// Resets the parser state, removing all tracked markers and zones.
    pub fn clear(&mut self) {
        self.markers.clear();
    }

    /// Get the raw markers list (for debugging).
    pub fn markers(&self) -> &[(Osc133Marker, u16, u16)] {
        &self.markers
    }
}

impl Default for Osc133Parser {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension trait for semantic zone testing.
///
/// Provides methods for working with OSC 133 semantic zones in terminal
/// testing scenarios. This trait is implemented for test harnesses that
/// support zone detection.
pub trait SemanticZoneExt {
    /// Get all detected semantic zones.
    ///
    /// Returns all zones that have been detected in the terminal output,
    /// including prompt, command, and output zones.
    fn zones(&self) -> IpcResult<Vec<SemanticZone>>;

    /// Get the zone at a specific position.
    ///
    /// # Arguments
    ///
    /// * `row` - Row position (0-indexed)
    /// * `col` - Column position (0-indexed)
    ///
    /// # Returns
    ///
    /// The zone at that position, or `None` if no zone contains that position.
    fn zone_at(&self, row: u16, col: u16) -> IpcResult<Option<SemanticZone>>;

    /// Get the last completed output zone.
    ///
    /// Returns the most recent output zone (from C to D marker).
    fn last_output_zone(&self) -> IpcResult<Option<SemanticZone>>;

    /// Get the last command zone.
    ///
    /// Returns the most recent command zone (from B to C marker).
    fn last_command_zone(&self) -> IpcResult<Option<SemanticZone>>;

    /// Extract text from a specific zone.
    ///
    /// # Arguments
    ///
    /// * `zone` - The zone to extract text from
    ///
    /// # Returns
    ///
    /// The text content within the zone boundaries.
    fn zone_text(&self, zone: &SemanticZone) -> IpcResult<String>;

    /// Assert a zone exists at the given position.
    ///
    /// # Arguments
    ///
    /// * `row` - Row position (0-indexed)
    /// * `col` - Column position (0-indexed)
    /// * `expected_type` - Expected zone type
    ///
    /// # Errors
    ///
    /// Returns an error if no zone exists at the position or the zone type doesn't match.
    fn assert_zone_at(&self, row: u16, col: u16, expected_type: ZoneType) -> IpcResult<()>;

    /// Wait for a new output zone to appear.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait
    ///
    /// # Returns
    ///
    /// The new output zone that appeared.
    ///
    /// # Errors
    ///
    /// Returns a timeout error if no new output zone appears within the timeout.
    fn wait_for_output_zone(&mut self, timeout: Duration) -> IpcResult<SemanticZone>;

    /// Wait for command completion (D marker).
    ///
    /// # Arguments
    ///
    /// * `timeout` - Maximum time to wait
    ///
    /// # Returns
    ///
    /// The exit code from the D marker, or `None` if no exit code was provided.
    ///
    /// # Errors
    ///
    /// Returns a timeout error if the command doesn't complete within the timeout.
    fn wait_for_command_complete(&mut self, timeout: Duration) -> IpcResult<Option<i32>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_osc133_marker_from_params() {
        assert_eq!(
            Osc133Marker::from_params("A"),
            Some(Osc133Marker::FreshLine)
        );
        assert_eq!(
            Osc133Marker::from_params("B"),
            Some(Osc133Marker::CommandStart)
        );
        assert_eq!(
            Osc133Marker::from_params("C"),
            Some(Osc133Marker::CommandExecuted)
        );
        assert_eq!(
            Osc133Marker::from_params("D"),
            Some(Osc133Marker::CommandFinished(None))
        );
        assert_eq!(
            Osc133Marker::from_params("D;0"),
            Some(Osc133Marker::CommandFinished(Some(0)))
        );
        assert_eq!(
            Osc133Marker::from_params("D;127"),
            Some(Osc133Marker::CommandFinished(Some(127)))
        );
        assert_eq!(Osc133Marker::from_params("X"), None);
    }

    #[test]
    fn test_parser_simple_sequence() {
        let mut parser = Osc133Parser::new();

        // Simulate: prompt marker, then command marker
        let data = b"\x1b]133;A\x07$ \x1b]133;B\x07ls\x1b]133;C\x07\nfile.txt\n\x1b]133;D;0\x07";
        parser.parse(data);

        let markers = parser.markers();
        assert_eq!(markers.len(), 4);
        assert_eq!(markers[0].0, Osc133Marker::FreshLine);
        assert_eq!(markers[1].0, Osc133Marker::CommandStart);
        assert_eq!(markers[2].0, Osc133Marker::CommandExecuted);
        assert_eq!(markers[3].0, Osc133Marker::CommandFinished(Some(0)));
    }

    #[test]
    fn test_parser_zones() {
        let mut parser = Osc133Parser::new();

        let data = b"\x1b]133;A\x07$ \x1b]133;B\x07ls\x1b]133;C\x07\nfile.txt\n\x1b]133;D;0\x07";
        parser.parse(data);

        let zones = parser.zones();
        assert_eq!(zones.len(), 3);

        // Prompt zone
        assert_eq!(zones[0].zone_type, ZoneType::Prompt);

        // Command zone
        assert_eq!(zones[1].zone_type, ZoneType::Command);

        // Output zone
        assert_eq!(zones[2].zone_type, ZoneType::Output);
        assert_eq!(zones[2].exit_code, Some(0));
    }

    #[test]
    fn test_parser_clear() {
        let mut parser = Osc133Parser::new();
        parser.parse(b"\x1b]133;A\x07\x1b]133;B\x07");
        assert_eq!(parser.markers().len(), 2);

        parser.clear();
        assert_eq!(parser.markers().len(), 0);
        assert_eq!(parser.zones().len(), 0);
    }

    #[test]
    fn test_parser_exit_codes() {
        let mut parser = Osc133Parser::new();

        // Success
        parser.parse(b"\x1b]133;C\x07output\x1b]133;D;0\x07");
        let zones = parser.zones();
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].exit_code, Some(0));

        parser.clear();

        // Failure
        parser.parse(b"\x1b]133;C\x07error\x1b]133;D;1\x07");
        let zones = parser.zones();
        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].exit_code, Some(1));
    }

    #[test]
    fn test_parser_st_terminator() {
        let mut parser = Osc133Parser::new();

        // Use ST (String Terminator) instead of BEL
        let data = b"\x1b]133;A\x1b\\$ \x1b]133;B\x1b\\";
        parser.parse(data);

        let markers = parser.markers();
        assert_eq!(markers.len(), 2);
        assert_eq!(markers[0].0, Osc133Marker::FreshLine);
        assert_eq!(markers[1].0, Osc133Marker::CommandStart);
    }

    #[test]
    fn test_semantic_zone_equality() {
        let zone1 = SemanticZone {
            zone_type: ZoneType::Prompt,
            start_row: 0,
            start_col: 0,
            end_row: 0,
            end_col: 5,
            exit_code: None,
        };

        let zone2 = SemanticZone {
            zone_type: ZoneType::Prompt,
            start_row: 0,
            start_col: 0,
            end_row: 0,
            end_col: 5,
            exit_code: None,
        };

        assert_eq!(zone1, zone2);
    }

    #[test]
    fn test_zone_types() {
        assert_ne!(ZoneType::Prompt, ZoneType::Command);
        assert_ne!(ZoneType::Command, ZoneType::Output);
        assert_ne!(ZoneType::Output, ZoneType::Prompt);
    }

    #[test]
    fn test_parser_default() {
        let parser = Osc133Parser::default();
        assert_eq!(parser.markers().len(), 0);
    }

    #[test]
    fn test_parser_newlines() {
        let mut parser = Osc133Parser::new();

        // Markers on different lines
        let data = b"\x1b]133;A\x07\n$ \x1b]133;B\x07\nls\n\x1b]133;C\x07";
        parser.parse(data);

        let markers = parser.markers();
        assert_eq!(markers.len(), 3);

        // Check that row tracking works
        assert_eq!(markers[0].1, 0); // First marker on row 0
        assert!(markers[1].1 >= 1); // Second marker on row 1 or later
        assert!(markers[2].1 >= 2); // Third marker on row 2 or later
    }
}

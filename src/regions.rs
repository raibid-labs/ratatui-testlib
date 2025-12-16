//! Fixed UI region testing helpers for status bars, tab bars, and window chrome.
//!
//! This module provides abstractions for testing fixed UI regions that overlay or
//! partition terminal content. Common use cases include:
//!
//! - **Status bars** at the top or bottom of the screen
//! - **Tab bars** for application navigation
//! - **Sidebars** for file trees or additional context
//! - **Window chrome** that surrounds the main content area
//!
//! # Overview
//!
//! When building TUI applications, it's common to have fixed regions that remain
//! constant while the main content area updates. Testing these requires:
//!
//! 1. Defining the fixed regions and their anchors
//! 2. Calculating the remaining content area after subtracting fixed regions
//! 3. Verifying that content appears in the correct region
//! 4. Testing resize behavior and region recalculation
//!
//! # Quick Start
//!
//! ```rust
//! use terminal_testlib::regions::{UiRegionTester, RegionBounds};
//!
//! // Define a terminal with fixed regions
//! let tester = UiRegionTester::new(80, 24)
//!     .with_status_bar(1)      // 1 row at bottom
//!     .with_tab_bar(2)         // 2 rows at top
//!     .with_left_sidebar(20);  // 20 cols on left
//!
//! // Get the content area after subtracting fixed regions
//! let content = tester.content_area();
//! assert_eq!(content.row, 2);     // Below tab bar
//! assert_eq!(content.col, 20);    // Right of sidebar
//! assert_eq!(content.height, 21); // 24 - 2 (tab) - 1 (status)
//! assert_eq!(content.width, 60);  // 80 - 20 (sidebar)
//!
//! // Check if a position is in a specific region
//! assert!(tester.is_in_region("status_bar", 23, 0));
//! assert!(tester.is_in_region("tab_bar", 0, 40));
//! assert!(tester.is_in_content_area(10, 40));
//! ```
//!
//! # Testing with ScarabTestHarness
//!
//! The `UiRegionTestExt` trait provides integration with `ScarabTestHarness`:
//!
//! ```rust,no_run
//! # #[cfg(feature = "scarab")]
//! # {
//! use std::time::Duration;
//! use terminal_testlib::{
//!     scarab::ScarabTestHarness,
//!     regions::{UiRegionTester, UiRegionTestExt},
//! };
//!
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! let mut harness = ScarabTestHarness::connect()?;
//! let tester = UiRegionTester::new(80, 24).with_status_bar(1);
//!
//! // Get contents of just the status bar
//! let status = harness.region_contents(&tester, "status_bar")?;
//! assert!(status.contains("Ready"));
//!
//! // Verify text doesn't appear in the status bar
//! harness.assert_not_in_region(&tester, "status_bar", "Error")?;
//!
//! // Test resize behavior
//! harness.verify_resize(&tester, 100, 30)?;
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! # Custom Regions
//!
//! For non-standard layouts, create custom regions:
//!
//! ```rust
//! use terminal_testlib::regions::{UiRegion, RegionAnchor, UiRegionTester};
//!
//! let custom = UiRegion {
//!     name: "notification_area".to_string(),
//!     anchor: RegionAnchor::Top,
//!     size: 3,
//! };
//!
//! let tester = UiRegionTester::new(80, 24).with_region(custom);
//! ```

use crate::ipc::{IpcError, IpcResult};

/// Defines a fixed UI region.
///
/// A UI region represents a fixed area of the terminal that doesn't change
/// with normal content updates. Regions are defined by an anchor point
/// (where they're positioned) and a size (in rows or columns).
///
/// # Examples
///
/// ```rust
/// use terminal_testlib::regions::{UiRegion, RegionAnchor};
///
/// // Status bar at bottom, 1 row tall
/// let status = UiRegion {
///     name: "status_bar".to_string(),
///     anchor: RegionAnchor::Bottom,
///     size: 1,
/// };
///
/// // Sidebar at left, 20 columns wide
/// let sidebar = UiRegion {
///     name: "sidebar".to_string(),
///     anchor: RegionAnchor::Left,
///     size: 20,
/// };
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UiRegion {
    /// Name of the region (e.g., "status_bar", "tab_bar").
    pub name: String,
    /// Where the region is anchored.
    pub anchor: RegionAnchor,
    /// Size in rows (for Top/Bottom anchors) or columns (for Left/Right anchors).
    pub size: u16,
}

/// Where a UI region is anchored.
///
/// Regions are anchored to one of the four edges of the terminal.
/// The anchor determines how the region's position is calculated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegionAnchor {
    /// Anchored to the top of the screen.
    Top,
    /// Anchored to the bottom of the screen.
    Bottom,
    /// Anchored to the left of the screen.
    Left,
    /// Anchored to the right of the screen.
    Right,
}

/// Rectangle bounds (row, col, width, height).
///
/// Represents the bounds of a region in terminal coordinate space.
/// This uses the standard terminal convention:
/// - `row` is the vertical position (0-indexed from top)
/// - `col` is the horizontal position (0-indexed from left)
/// - `width` is the number of columns
/// - `height` is the number of rows
///
/// # Examples
///
/// ```rust
/// use terminal_testlib::regions::RegionBounds;
///
/// let bounds = RegionBounds::new(5, 10, 60, 18);
///
/// // Check if a position is within bounds
/// assert!(bounds.contains(10, 20));
/// assert!(!bounds.contains(0, 0));
///
/// // Check for intersection with another region
/// let other = RegionBounds::new(3, 8, 20, 10);
/// assert!(bounds.intersects(&other));
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegionBounds {
    /// Starting row (0-indexed).
    pub row: u16,
    /// Starting column (0-indexed).
    pub col: u16,
    /// Width in columns.
    pub width: u16,
    /// Height in rows.
    pub height: u16,
}

impl RegionBounds {
    /// Create new region bounds.
    ///
    /// # Arguments
    ///
    /// * `row` - Starting row (0-indexed)
    /// * `col` - Starting column (0-indexed)
    /// * `width` - Width in columns
    /// * `height` - Height in rows
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::RegionBounds;
    ///
    /// let bounds = RegionBounds::new(0, 0, 80, 24);
    /// assert_eq!(bounds.row, 0);
    /// assert_eq!(bounds.col, 0);
    /// assert_eq!(bounds.width, 80);
    /// assert_eq!(bounds.height, 24);
    /// ```
    pub const fn new(row: u16, col: u16, width: u16, height: u16) -> Self {
        Self {
            row,
            col,
            width,
            height,
        }
    }

    /// Check if a position (row, col) is within this region.
    ///
    /// # Arguments
    ///
    /// * `row` - Row to check
    /// * `col` - Column to check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::RegionBounds;
    ///
    /// let bounds = RegionBounds::new(10, 20, 40, 5);
    ///
    /// assert!(bounds.contains(10, 20));  // Top-left corner
    /// assert!(bounds.contains(12, 30));  // Inside
    /// assert!(!bounds.contains(5, 25));  // Above
    /// assert!(!bounds.contains(15, 25)); // Below
    /// ```
    pub const fn contains(&self, row: u16, col: u16) -> bool {
        row >= self.row
            && row < self.row.saturating_add(self.height)
            && col >= self.col
            && col < self.col.saturating_add(self.width)
    }

    /// Check if this region intersects with another region.
    ///
    /// # Arguments
    ///
    /// * `other` - The other region to check for intersection
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::RegionBounds;
    ///
    /// let a = RegionBounds::new(0, 0, 10, 10);
    /// let b = RegionBounds::new(5, 5, 10, 10);  // Overlaps
    /// let c = RegionBounds::new(20, 20, 10, 10); // Separate
    ///
    /// assert!(a.intersects(&b));
    /// assert!(!a.intersects(&c));
    /// ```
    pub const fn intersects(&self, other: &RegionBounds) -> bool {
        // Check if the rectangles don't overlap, then negate
        !(self.row.saturating_add(self.height) <= other.row
            || other.row.saturating_add(other.height) <= self.row
            || self.col.saturating_add(self.width) <= other.col
            || other.col.saturating_add(other.width) <= self.col)
    }

    /// Get the bottom row (exclusive).
    #[inline]
    const fn bottom(&self) -> u16 {
        self.row.saturating_add(self.height)
    }

    /// Get the right column (exclusive).
    #[inline]
    const fn right(&self) -> u16 {
        self.col.saturating_add(self.width)
    }
}

/// UI region tester for verifying fixed regions.
///
/// This struct manages a collection of fixed UI regions and provides
/// methods to query region bounds, check positions, and calculate the
/// remaining content area.
///
/// Regions are applied in the order: Top, Bottom, Left, Right. This means
/// that if you have both a top bar and a left sidebar, the sidebar will
/// start below the top bar.
///
/// # Examples
///
/// ```rust
/// use terminal_testlib::regions::{UiRegionTester, RegionBounds};
///
/// let tester = UiRegionTester::new(80, 24)
///     .with_status_bar(1)
///     .with_tab_bar(2);
///
/// // Get bounds for a specific region
/// let status = tester.region_bounds("status_bar").unwrap();
/// assert_eq!(status.row, 23);  // Bottom row
/// assert_eq!(status.height, 1);
///
/// // Calculate content area
/// let content = tester.content_area();
/// assert_eq!(content.row, 2);      // Below tab bar
/// assert_eq!(content.height, 21);  // 24 - 2 (tab) - 1 (status)
/// ```
#[derive(Debug, Clone)]
pub struct UiRegionTester {
    regions: Vec<UiRegion>,
    screen_width: u16,
    screen_height: u16,
}

impl UiRegionTester {
    /// Create a new tester with screen dimensions.
    ///
    /// # Arguments
    ///
    /// * `width` - Screen width in columns
    /// * `height` - Screen height in rows
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::UiRegionTester;
    ///
    /// let tester = UiRegionTester::new(80, 24);
    /// assert_eq!(tester.screen_dimensions(), (80, 24));
    /// ```
    pub fn new(width: u16, height: u16) -> Self {
        Self {
            regions: Vec::new(),
            screen_width: width,
            screen_height: height,
        }
    }

    /// Get the screen dimensions (width, height).
    pub fn screen_dimensions(&self) -> (u16, u16) {
        (self.screen_width, self.screen_height)
    }

    /// Add a status bar region at the bottom.
    ///
    /// # Arguments
    ///
    /// * `height` - Height of the status bar in rows
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::UiRegionTester;
    ///
    /// let tester = UiRegionTester::new(80, 24).with_status_bar(1);
    /// let bounds = tester.region_bounds("status_bar").unwrap();
    /// assert_eq!(bounds.row, 23);
    /// assert_eq!(bounds.height, 1);
    /// ```
    pub fn with_status_bar(mut self, height: u16) -> Self {
        self.regions.push(UiRegion {
            name: "status_bar".to_string(),
            anchor: RegionAnchor::Bottom,
            size: height,
        });
        self
    }

    /// Add a tab bar region at the top.
    ///
    /// # Arguments
    ///
    /// * `height` - Height of the tab bar in rows
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::UiRegionTester;
    ///
    /// let tester = UiRegionTester::new(80, 24).with_tab_bar(2);
    /// let bounds = tester.region_bounds("tab_bar").unwrap();
    /// assert_eq!(bounds.row, 0);
    /// assert_eq!(bounds.height, 2);
    /// ```
    pub fn with_tab_bar(mut self, height: u16) -> Self {
        self.regions.push(UiRegion {
            name: "tab_bar".to_string(),
            anchor: RegionAnchor::Top,
            size: height,
        });
        self
    }

    /// Add a left sidebar region.
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the sidebar in columns
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::UiRegionTester;
    ///
    /// let tester = UiRegionTester::new(80, 24).with_left_sidebar(20);
    /// let bounds = tester.region_bounds("left_sidebar").unwrap();
    /// assert_eq!(bounds.col, 0);
    /// assert_eq!(bounds.width, 20);
    /// ```
    pub fn with_left_sidebar(mut self, width: u16) -> Self {
        self.regions.push(UiRegion {
            name: "left_sidebar".to_string(),
            anchor: RegionAnchor::Left,
            size: width,
        });
        self
    }

    /// Add a right sidebar region.
    ///
    /// # Arguments
    ///
    /// * `width` - Width of the sidebar in columns
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::UiRegionTester;
    ///
    /// let tester = UiRegionTester::new(80, 24).with_right_sidebar(15);
    /// let bounds = tester.region_bounds("right_sidebar").unwrap();
    /// assert_eq!(bounds.col, 65);  // 80 - 15
    /// assert_eq!(bounds.width, 15);
    /// ```
    pub fn with_right_sidebar(mut self, width: u16) -> Self {
        self.regions.push(UiRegion {
            name: "right_sidebar".to_string(),
            anchor: RegionAnchor::Right,
            size: width,
        });
        self
    }

    /// Add a custom region.
    ///
    /// # Arguments
    ///
    /// * `region` - The custom region to add
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::{UiRegion, RegionAnchor, UiRegionTester};
    ///
    /// let custom = UiRegion {
    ///     name: "notification".to_string(),
    ///     anchor: RegionAnchor::Top,
    ///     size: 3,
    /// };
    ///
    /// let tester = UiRegionTester::new(80, 24).with_region(custom);
    /// assert!(tester.region_bounds("notification").is_some());
    /// ```
    pub fn with_region(mut self, region: UiRegion) -> Self {
        self.regions.push(region);
        self
    }

    /// Get bounds for a named region.
    ///
    /// Returns `None` if the region doesn't exist.
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the region to look up
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::UiRegionTester;
    ///
    /// let tester = UiRegionTester::new(80, 24).with_status_bar(1);
    ///
    /// assert!(tester.region_bounds("status_bar").is_some());
    /// assert!(tester.region_bounds("nonexistent").is_none());
    /// ```
    pub fn region_bounds(&self, name: &str) -> Option<RegionBounds> {
        let region = self.regions.iter().find(|r| r.name == name)?;
        Some(self.calculate_bounds(region))
    }

    /// Calculate bounds for a region based on current screen dimensions.
    fn calculate_bounds(&self, region: &UiRegion) -> RegionBounds {
        // First, calculate occupied space by other regions to determine offsets
        let mut top_offset = 0u16;
        let mut bottom_offset = 0u16;
        let mut left_offset = 0u16;
        let mut right_offset = 0u16;

        // Calculate offsets from regions that come before this one
        for r in &self.regions {
            if std::ptr::eq(r, region) {
                break;
            }
            match r.anchor {
                RegionAnchor::Top => top_offset = top_offset.saturating_add(r.size),
                RegionAnchor::Bottom => bottom_offset = bottom_offset.saturating_add(r.size),
                RegionAnchor::Left => left_offset = left_offset.saturating_add(r.size),
                RegionAnchor::Right => right_offset = right_offset.saturating_add(r.size),
            }
        }

        match region.anchor {
            RegionAnchor::Top => RegionBounds::new(
                top_offset,
                0,
                self.screen_width,
                region.size.min(self.screen_height.saturating_sub(top_offset)),
            ),
            RegionAnchor::Bottom => {
                let row = self
                    .screen_height
                    .saturating_sub(region.size)
                    .saturating_sub(bottom_offset);
                RegionBounds::new(row, 0, self.screen_width, region.size)
            }
            RegionAnchor::Left => {
                let available_height = self
                    .screen_height
                    .saturating_sub(top_offset)
                    .saturating_sub(bottom_offset);
                RegionBounds::new(top_offset, left_offset, region.size, available_height)
            }
            RegionAnchor::Right => {
                let available_height = self
                    .screen_height
                    .saturating_sub(top_offset)
                    .saturating_sub(bottom_offset);
                let col = self
                    .screen_width
                    .saturating_sub(region.size)
                    .saturating_sub(right_offset);
                RegionBounds::new(top_offset, col, region.size, available_height)
            }
        }
    }

    /// Calculate the terminal content area after subtracting fixed regions.
    ///
    /// The content area is the remaining space after all fixed regions are
    /// accounted for. This is where the main application content should be drawn.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::UiRegionTester;
    ///
    /// let tester = UiRegionTester::new(80, 24)
    ///     .with_tab_bar(2)
    ///     .with_status_bar(1)
    ///     .with_left_sidebar(20);
    ///
    /// let content = tester.content_area();
    /// assert_eq!(content.row, 2);      // Below tab bar
    /// assert_eq!(content.col, 20);     // Right of sidebar
    /// assert_eq!(content.height, 21);  // 24 - 2 - 1
    /// assert_eq!(content.width, 60);   // 80 - 20
    /// ```
    pub fn content_area(&self) -> RegionBounds {
        let mut top = 0u16;
        let mut bottom = 0u16;
        let mut left = 0u16;
        let mut right = 0u16;

        for region in &self.regions {
            match region.anchor {
                RegionAnchor::Top => top = top.saturating_add(region.size),
                RegionAnchor::Bottom => bottom = bottom.saturating_add(region.size),
                RegionAnchor::Left => left = left.saturating_add(region.size),
                RegionAnchor::Right => right = right.saturating_add(region.size),
            }
        }

        let height = self.screen_height.saturating_sub(top).saturating_sub(bottom);
        let width = self.screen_width.saturating_sub(left).saturating_sub(right);

        RegionBounds::new(top, left, width, height)
    }

    /// Check if a position is in the content area (not in any fixed region).
    ///
    /// # Arguments
    ///
    /// * `row` - Row to check
    /// * `col` - Column to check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::UiRegionTester;
    ///
    /// let tester = UiRegionTester::new(80, 24).with_status_bar(1);
    ///
    /// assert!(tester.is_in_content_area(10, 10));
    /// assert!(!tester.is_in_content_area(23, 10));  // Status bar row
    /// ```
    pub fn is_in_content_area(&self, row: u16, col: u16) -> bool {
        let content = self.content_area();
        content.contains(row, col)
    }

    /// Check if a position is in a specific region.
    ///
    /// # Arguments
    ///
    /// * `region_name` - Name of the region to check
    /// * `row` - Row to check
    /// * `col` - Column to check
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::UiRegionTester;
    ///
    /// let tester = UiRegionTester::new(80, 24).with_status_bar(1);
    ///
    /// assert!(tester.is_in_region("status_bar", 23, 0));
    /// assert!(!tester.is_in_region("status_bar", 0, 0));
    /// ```
    pub fn is_in_region(&self, region_name: &str, row: u16, col: u16) -> bool {
        self.region_bounds(region_name)
            .map(|bounds| bounds.contains(row, col))
            .unwrap_or(false)
    }

    /// Get a list of all region names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use terminal_testlib::regions::UiRegionTester;
    ///
    /// let tester = UiRegionTester::new(80, 24)
    ///     .with_status_bar(1)
    ///     .with_tab_bar(2);
    ///
    /// let names = tester.region_names();
    /// assert_eq!(names.len(), 2);
    /// assert!(names.contains(&"status_bar".to_string()));
    /// assert!(names.contains(&"tab_bar".to_string()));
    /// ```
    pub fn region_names(&self) -> Vec<String> {
        self.regions.iter().map(|r| r.name.clone()).collect()
    }
}

/// Extension trait for UI region testing with harnesses.
///
/// This trait integrates UI region testing with test harnesses like
/// `ScarabTestHarness`. It provides methods to extract region contents,
/// verify region constraints, and test resize behavior.
#[cfg(feature = "scarab")]
pub trait UiRegionTestExt {
    /// Get the grid contents for a specific region.
    ///
    /// This method extracts only the portion of the terminal grid that
    /// falls within the specified region.
    ///
    /// # Arguments
    ///
    /// * `tester` - The UI region tester with region definitions
    /// * `region_name` - Name of the region to extract
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The region doesn't exist
    /// - Failed to read the terminal grid
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "scarab")]
    /// # {
    /// use terminal_testlib::{
    ///     scarab::ScarabTestHarness,
    ///     regions::{UiRegionTester, UiRegionTestExt},
    /// };
    ///
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut harness = ScarabTestHarness::connect()?;
    /// let tester = UiRegionTester::new(80, 24).with_status_bar(1);
    ///
    /// let status = harness.region_contents(&tester, "status_bar")?;
    /// assert!(status.contains("Ready"));
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn region_contents(&self, tester: &UiRegionTester, region_name: &str) -> IpcResult<String>;

    /// Get the content area grid contents (excluding fixed regions).
    ///
    /// This method extracts the terminal grid content that falls within
    /// the content area (i.e., not in any fixed regions).
    ///
    /// # Arguments
    ///
    /// * `tester` - The UI region tester with region definitions
    ///
    /// # Errors
    ///
    /// Returns an error if failed to read the terminal grid.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "scarab")]
    /// # {
    /// use terminal_testlib::{
    ///     scarab::ScarabTestHarness,
    ///     regions::{UiRegionTester, UiRegionTestExt},
    /// };
    ///
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut harness = ScarabTestHarness::connect()?;
    /// let tester = UiRegionTester::new(80, 24)
    ///     .with_status_bar(1)
    ///     .with_tab_bar(2);
    ///
    /// let content = harness.content_area_contents(&tester)?;
    /// assert!(content.contains("Main content"));
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn content_area_contents(&self, tester: &UiRegionTester) -> IpcResult<String>;

    /// Assert that text doesn't appear in a fixed region.
    ///
    /// This is useful for verifying that error messages or other content
    /// doesn't leak into UI chrome regions.
    ///
    /// # Arguments
    ///
    /// * `tester` - The UI region tester with region definitions
    /// * `region_name` - Name of the region to check
    /// * `text` - Text that should not appear in the region
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The region doesn't exist
    /// - Failed to read the terminal grid
    /// - The text is found in the region
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "scarab")]
    /// # {
    /// use terminal_testlib::{
    ///     scarab::ScarabTestHarness,
    ///     regions::{UiRegionTester, UiRegionTestExt},
    /// };
    ///
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut harness = ScarabTestHarness::connect()?;
    /// let tester = UiRegionTester::new(80, 24).with_status_bar(1);
    ///
    /// // Verify error messages don't leak into status bar
    /// harness.assert_not_in_region(&tester, "status_bar", "ERROR")?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn assert_not_in_region(
        &self,
        tester: &UiRegionTester,
        region_name: &str,
        text: &str,
    ) -> IpcResult<()>;

    /// Assert that content in a fixed region matches expected.
    ///
    /// # Arguments
    ///
    /// * `tester` - The UI region tester with region definitions
    /// * `region_name` - Name of the region to check
    /// * `expected` - Expected text in the region
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The region doesn't exist
    /// - Failed to read the terminal grid
    /// - The expected text is not found in the region
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "scarab")]
    /// # {
    /// use terminal_testlib::{
    ///     scarab::ScarabTestHarness,
    ///     regions::{UiRegionTester, UiRegionTestExt},
    /// };
    ///
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut harness = ScarabTestHarness::connect()?;
    /// let tester = UiRegionTester::new(80, 24).with_status_bar(1);
    ///
    /// harness.assert_region_contains(&tester, "status_bar", "Ready")?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn assert_region_contains(
        &self,
        tester: &UiRegionTester,
        region_name: &str,
        expected: &str,
    ) -> IpcResult<()>;

    /// Verify resize event correctly calculated terminal dimensions.
    ///
    /// This method resizes the terminal and verifies that the content area
    /// and fixed regions are recalculated correctly.
    ///
    /// # Arguments
    ///
    /// * `tester` - The UI region tester with region definitions (will be updated)
    /// * `new_width` - New terminal width
    /// * `new_height` - New terminal height
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to resize the terminal
    /// - The terminal dimensions don't match after resize
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # #[cfg(feature = "scarab")]
    /// # {
    /// use terminal_testlib::{
    ///     scarab::ScarabTestHarness,
    ///     regions::{UiRegionTester, UiRegionTestExt},
    /// };
    ///
    /// # fn test() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut harness = ScarabTestHarness::connect()?;
    /// let mut tester = UiRegionTester::new(80, 24).with_status_bar(1);
    ///
    /// harness.verify_resize(&mut tester, 100, 30)?;
    /// # Ok(())
    /// # }
    /// # }
    /// ```
    fn verify_resize(
        &mut self,
        tester: &mut UiRegionTester,
        new_width: u16,
        new_height: u16,
    ) -> IpcResult<()>;
}

#[cfg(feature = "scarab")]
impl UiRegionTestExt for crate::scarab::ScarabTestHarness {
    fn region_contents(&self, tester: &UiRegionTester, region_name: &str) -> IpcResult<String> {
        let bounds = tester.region_bounds(region_name).ok_or_else(|| {
            IpcError::InvalidData(format!("Region '{}' not found", region_name))
        })?;

        let full_grid = self.grid_contents()?;
        extract_region_from_grid(&full_grid, tester.screen_width, &bounds)
    }

    fn content_area_contents(&self, tester: &UiRegionTester) -> IpcResult<String> {
        let bounds = tester.content_area();
        let full_grid = self.grid_contents()?;
        extract_region_from_grid(&full_grid, tester.screen_width, &bounds)
    }

    fn assert_not_in_region(
        &self,
        tester: &UiRegionTester,
        region_name: &str,
        text: &str,
    ) -> IpcResult<()> {
        let region_content = self.region_contents(tester, region_name)?;

        if region_content.contains(text) {
            return Err(IpcError::InvalidData(format!(
                "Text '{}' found in region '{}' but should not be present.\nRegion content:\n{}",
                text, region_name, region_content
            )));
        }

        Ok(())
    }

    fn assert_region_contains(
        &self,
        tester: &UiRegionTester,
        region_name: &str,
        expected: &str,
    ) -> IpcResult<()> {
        let region_content = self.region_contents(tester, region_name)?;

        if !region_content.contains(expected) {
            return Err(IpcError::InvalidData(format!(
                "Expected text '{}' not found in region '{}'.\nRegion content:\n{}",
                expected, region_name, region_content
            )));
        }

        Ok(())
    }

    fn verify_resize(
        &mut self,
        tester: &mut UiRegionTester,
        new_width: u16,
        new_height: u16,
    ) -> IpcResult<()> {
        // Send resize request
        self.resize(new_width, new_height)?;

        // Wait briefly for the resize to take effect
        std::thread::sleep(std::time::Duration::from_millis(100));

        // Refresh the shared memory to get updated dimensions
        self.refresh()?;

        // Verify the dimensions
        let (actual_width, actual_height) = self.dimensions();
        if actual_width != new_width || actual_height != new_height {
            return Err(IpcError::InvalidData(format!(
                "Resize verification failed: expected {}x{}, got {}x{}",
                new_width, new_height, actual_width, actual_height
            )));
        }

        // Update the tester's dimensions
        tester.screen_width = new_width;
        tester.screen_height = new_height;

        Ok(())
    }
}

/// Helper function to extract a region from the full grid.
#[cfg(feature = "scarab")]
fn extract_region_from_grid(
    grid: &str,
    screen_width: u16,
    bounds: &RegionBounds,
) -> IpcResult<String> {
    let lines: Vec<&str> = grid.lines().collect();

    let mut result = String::new();
    let start_row = bounds.row as usize;
    let end_row = (bounds.row + bounds.height) as usize;
    let start_col = bounds.col as usize;
    let end_col = (bounds.col + bounds.width) as usize;

    for row_idx in start_row..end_row {
        if row_idx >= lines.len() {
            break;
        }

        let line = lines[row_idx];
        let chars: Vec<char> = line.chars().collect();

        if row_idx > start_row {
            result.push('\n');
        }

        // Extract the column range
        for col_idx in start_col..end_col.min(chars.len().max(screen_width as usize)) {
            if col_idx < chars.len() {
                result.push(chars[col_idx]);
            } else {
                result.push(' ');
            }
        }
    }

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_bounds_new() {
        let bounds = RegionBounds::new(5, 10, 20, 15);
        assert_eq!(bounds.row, 5);
        assert_eq!(bounds.col, 10);
        assert_eq!(bounds.width, 20);
        assert_eq!(bounds.height, 15);
    }

    #[test]
    fn test_region_bounds_contains() {
        let bounds = RegionBounds::new(10, 20, 40, 5);

        // Inside
        assert!(bounds.contains(10, 20)); // Top-left
        assert!(bounds.contains(12, 30)); // Middle
        assert!(bounds.contains(14, 59)); // Bottom-right corner

        // Outside
        assert!(!bounds.contains(9, 25)); // Above
        assert!(!bounds.contains(15, 25)); // Below
        assert!(!bounds.contains(12, 19)); // Left
        assert!(!bounds.contains(12, 60)); // Right
    }

    #[test]
    fn test_region_bounds_intersects() {
        let a = RegionBounds::new(0, 0, 10, 10);
        let b = RegionBounds::new(5, 5, 10, 10); // Overlaps
        let c = RegionBounds::new(20, 20, 10, 10); // Separate
        let d = RegionBounds::new(8, 8, 5, 5); // Contained

        assert!(a.intersects(&b));
        assert!(b.intersects(&a));
        assert!(!a.intersects(&c));
        assert!(!c.intersects(&a));
        assert!(a.intersects(&d));
        assert!(d.intersects(&a));
    }

    #[test]
    fn test_ui_region_tester_new() {
        let tester = UiRegionTester::new(80, 24);
        assert_eq!(tester.screen_dimensions(), (80, 24));
        assert_eq!(tester.region_names().len(), 0);
    }

    #[test]
    fn test_status_bar() {
        let tester = UiRegionTester::new(80, 24).with_status_bar(1);

        let bounds = tester.region_bounds("status_bar").unwrap();
        assert_eq!(bounds.row, 23);
        assert_eq!(bounds.col, 0);
        assert_eq!(bounds.width, 80);
        assert_eq!(bounds.height, 1);
    }

    #[test]
    fn test_tab_bar() {
        let tester = UiRegionTester::new(80, 24).with_tab_bar(2);

        let bounds = tester.region_bounds("tab_bar").unwrap();
        assert_eq!(bounds.row, 0);
        assert_eq!(bounds.col, 0);
        assert_eq!(bounds.width, 80);
        assert_eq!(bounds.height, 2);
    }

    #[test]
    fn test_left_sidebar() {
        let tester = UiRegionTester::new(80, 24).with_left_sidebar(20);

        let bounds = tester.region_bounds("left_sidebar").unwrap();
        assert_eq!(bounds.row, 0);
        assert_eq!(bounds.col, 0);
        assert_eq!(bounds.width, 20);
        assert_eq!(bounds.height, 24);
    }

    #[test]
    fn test_right_sidebar() {
        let tester = UiRegionTester::new(80, 24).with_right_sidebar(15);

        let bounds = tester.region_bounds("right_sidebar").unwrap();
        assert_eq!(bounds.row, 0);
        assert_eq!(bounds.col, 65); // 80 - 15
        assert_eq!(bounds.width, 15);
        assert_eq!(bounds.height, 24);
    }

    #[test]
    fn test_combined_regions() {
        let tester = UiRegionTester::new(80, 24)
            .with_tab_bar(2)
            .with_status_bar(1)
            .with_left_sidebar(20);

        // Tab bar should be at top
        let tab_bounds = tester.region_bounds("tab_bar").unwrap();
        assert_eq!(tab_bounds.row, 0);
        assert_eq!(tab_bounds.height, 2);

        // Status bar should be at bottom
        let status_bounds = tester.region_bounds("status_bar").unwrap();
        assert_eq!(status_bounds.row, 23);
        assert_eq!(status_bounds.height, 1);

        // Sidebar should be between tab and status
        let sidebar_bounds = tester.region_bounds("left_sidebar").unwrap();
        assert_eq!(sidebar_bounds.row, 2); // Below tab bar
        assert_eq!(sidebar_bounds.height, 21); // 24 - 2 (tab) - 1 (status)
    }

    #[test]
    fn test_content_area() {
        let tester = UiRegionTester::new(80, 24)
            .with_tab_bar(2)
            .with_status_bar(1)
            .with_left_sidebar(20);

        let content = tester.content_area();
        assert_eq!(content.row, 2); // Below tab bar
        assert_eq!(content.col, 20); // Right of sidebar
        assert_eq!(content.height, 21); // 24 - 2 (tab) - 1 (status)
        assert_eq!(content.width, 60); // 80 - 20 (sidebar)
    }

    #[test]
    fn test_is_in_content_area() {
        let tester = UiRegionTester::new(80, 24)
            .with_tab_bar(2)
            .with_status_bar(1);

        // In content area
        assert!(tester.is_in_content_area(10, 10));
        assert!(tester.is_in_content_area(2, 0)); // First content row

        // In fixed regions
        assert!(!tester.is_in_content_area(0, 10)); // Tab bar
        assert!(!tester.is_in_content_area(1, 10)); // Tab bar
        assert!(!tester.is_in_content_area(23, 10)); // Status bar
    }

    #[test]
    fn test_is_in_region() {
        let tester = UiRegionTester::new(80, 24)
            .with_status_bar(1)
            .with_tab_bar(2);

        assert!(tester.is_in_region("status_bar", 23, 0));
        assert!(tester.is_in_region("status_bar", 23, 79));
        assert!(!tester.is_in_region("status_bar", 22, 0));

        assert!(tester.is_in_region("tab_bar", 0, 0));
        assert!(tester.is_in_region("tab_bar", 1, 40));
        assert!(!tester.is_in_region("tab_bar", 2, 0));
    }

    #[test]
    fn test_region_names() {
        let tester = UiRegionTester::new(80, 24)
            .with_status_bar(1)
            .with_tab_bar(2)
            .with_left_sidebar(20);

        let names = tester.region_names();
        assert_eq!(names.len(), 3);
        assert!(names.contains(&"status_bar".to_string()));
        assert!(names.contains(&"tab_bar".to_string()));
        assert!(names.contains(&"left_sidebar".to_string()));
    }

    #[test]
    fn test_custom_region() {
        let custom = UiRegion {
            name: "notification".to_string(),
            anchor: RegionAnchor::Top,
            size: 3,
        };

        let tester = UiRegionTester::new(80, 24).with_region(custom);

        let bounds = tester.region_bounds("notification").unwrap();
        assert_eq!(bounds.row, 0);
        assert_eq!(bounds.height, 3);
    }

    #[test]
    fn test_multiple_same_anchor() {
        let custom1 = UiRegion {
            name: "header".to_string(),
            anchor: RegionAnchor::Top,
            size: 1,
        };
        let custom2 = UiRegion {
            name: "tabs".to_string(),
            anchor: RegionAnchor::Top,
            size: 2,
        };

        let tester = UiRegionTester::new(80, 24)
            .with_region(custom1)
            .with_region(custom2);

        let header = tester.region_bounds("header").unwrap();
        assert_eq!(header.row, 0);
        assert_eq!(header.height, 1);

        let tabs = tester.region_bounds("tabs").unwrap();
        assert_eq!(tabs.row, 1); // Below header
        assert_eq!(tabs.height, 2);
    }

    #[test]
    fn test_empty_content_area() {
        // Create a tester where regions consume entire screen
        let tester = UiRegionTester::new(80, 24)
            .with_tab_bar(12)
            .with_status_bar(12);

        let content = tester.content_area();
        assert_eq!(content.height, 0);
    }

    #[test]
    fn test_region_bounds_edge_cases() {
        let bounds = RegionBounds::new(0, 0, 1, 1);

        // Single cell
        assert!(bounds.contains(0, 0));
        assert!(!bounds.contains(0, 1));
        assert!(!bounds.contains(1, 0));
    }
}

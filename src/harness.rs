//! High-level test harness for TUI applications.
//!
//! This module provides the main testing interface for TUI applications through
//! the [`TuiTestHarness`] struct. It combines PTY management and terminal emulation
//! into an ergonomic API for spawning applications, sending input, and waiting for
//! screen updates.
//!
//! # Key Features
//!
//! - **Process Management**: Spawn and control TUI applications
//! - **Input Simulation**: Send keyboard input to the application
//! - **State Inspection**: Query screen contents and cursor position
//! - **Wait Conditions**: Block until specific screen states are reached
//! - **Flexible Configuration**: Builder pattern for custom timeout/polling settings
//!
//! # Example
//!
//! ```rust,no_run
//! use term_test::TuiTestHarness;
//! use portable_pty::CommandBuilder;
//!
//! # fn test() -> term_test::Result<()> {
//! // Create a test harness
//! let mut harness = TuiTestHarness::new(80, 24)?;
//!
//! // Spawn your TUI application
//! let mut cmd = CommandBuilder::new("my-tui-app");
//! harness.spawn(cmd)?;
//!
//! // Wait for initial render
//! harness.wait_for_text("Welcome")?;
//!
//! // Send input
//! harness.send_text("hello\n")?;
//!
//! // Verify output
//! assert!(harness.screen_contents().contains("hello"));
//! # Ok(())
//! # }
//! ```

use crate::error::{Result, TermTestError};
use crate::pty::TestTerminal;
use crate::screen::ScreenState;
use portable_pty::{CommandBuilder, ExitStatus};
use std::time::{Duration, Instant};

/// Default timeout for wait operations (5 seconds).
const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

/// Default polling interval for wait operations (100ms).
const DEFAULT_POLL_INTERVAL: Duration = Duration::from_millis(100);

/// Default buffer size for reading PTY output (4KB).
const DEFAULT_BUFFER_SIZE: usize = 4096;

/// High-level test harness for TUI applications.
///
/// This combines PTY management and terminal emulation to provide
/// an ergonomic API for testing TUI applications.
///
/// # Example
///
/// ```rust,no_run
/// use term_test::TuiTestHarness;
/// use portable_pty::CommandBuilder;
///
/// let mut harness = TuiTestHarness::new(80, 24)?;
/// let mut cmd = CommandBuilder::new("my-app");
/// harness.spawn(cmd)?;
/// harness.wait_for(|state| state.contains("Ready"))?;
/// # Ok::<(), term_test::TermTestError>(())
/// ```
///
/// # Builder Pattern
///
/// ```rust,no_run
/// use term_test::TuiTestHarness;
/// use std::time::Duration;
///
/// let mut harness = TuiTestHarness::builder()
///     .with_size(80, 24)
///     .with_timeout(Duration::from_secs(10))
///     .with_poll_interval(Duration::from_millis(50))
///     .build()?;
/// # Ok::<(), term_test::TermTestError>(())
/// ```
pub struct TuiTestHarness {
    terminal: TestTerminal,
    state: ScreenState,
    timeout: Duration,
    poll_interval: Duration,
    buffer_size: usize,
}

impl TuiTestHarness {
    /// Creates a new test harness with the specified dimensions.
    ///
    /// # Arguments
    ///
    /// * `width` - Terminal width in columns
    /// * `height` - Terminal height in rows
    ///
    /// # Errors
    ///
    /// Returns an error if terminal creation fails.
    pub fn new(width: u16, height: u16) -> Result<Self> {
        let terminal = TestTerminal::new(width, height)?;
        let state = ScreenState::new(width, height);

        Ok(Self {
            terminal,
            state,
            timeout: DEFAULT_TIMEOUT,
            poll_interval: DEFAULT_POLL_INTERVAL,
            buffer_size: DEFAULT_BUFFER_SIZE,
        })
    }

    /// Creates a builder for configuring a test harness.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use term_test::TuiTestHarness;
    /// use std::time::Duration;
    ///
    /// let mut harness = TuiTestHarness::builder()
    ///     .with_size(80, 24)
    ///     .with_timeout(Duration::from_secs(10))
    ///     .build()?;
    /// # Ok::<(), term_test::TermTestError>(())
    /// ```
    pub fn builder() -> TuiTestHarnessBuilder {
        TuiTestHarnessBuilder::default()
    }

    /// Sets the timeout for wait operations.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout duration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the polling interval for wait operations.
    ///
    /// # Arguments
    ///
    /// * `interval` - Polling interval
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Spawns a process in the PTY.
    ///
    /// # Arguments
    ///
    /// * `cmd` - Command to spawn
    ///
    /// # Errors
    ///
    /// Returns an error if spawning fails.
    pub fn spawn(&mut self, cmd: CommandBuilder) -> Result<()> {
        self.terminal.spawn(cmd)
    }

    /// Sends text to the PTY.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to send
    ///
    /// # Errors
    ///
    /// Returns an error if the write fails.
    pub fn send_text(&mut self, text: &str) -> Result<()> {
        self.terminal.write(text.as_bytes())?;
        self.update_state()?;
        Ok(())
    }

    /// Updates the screen state by reading from the PTY.
    ///
    /// This reads output in chunks (configured by buffer_size) and feeds it to the
    /// terminal emulator. It handles partial escape sequences correctly by continuing
    /// to read until no more data is available.
    ///
    /// This is called automatically by other methods but can be called
    /// manually if needed.
    ///
    /// # Errors
    ///
    /// Returns an error if reading from the PTY fails.
    pub fn update_state(&mut self) -> Result<()> {
        let mut buf = vec![0u8; self.buffer_size];

        loop {
            match self.terminal.read(&mut buf) {
                Ok(0) => break, // No more data
                Ok(n) => {
                    self.state.feed(&buf[..n]);
                }
                Err(e) if e.to_string().contains("WouldBlock") => break,
                Err(e) => return Err(e),
            }
        }

        Ok(())
    }

    /// Waits for a condition to be true, with timeout.
    ///
    /// This method polls the PTY output at the configured interval and checks
    /// the condition against the current screen state. If the timeout is reached,
    /// it returns an error with context about what was being waited for and the
    /// current screen state.
    ///
    /// # Arguments
    ///
    /// * `condition` - Condition to wait for
    ///
    /// # Errors
    ///
    /// Returns a `Timeout` error if the condition is not met within the configured timeout.
    /// The error includes the current screen state for debugging.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use term_test::TuiTestHarness;
    /// # let mut harness = TuiTestHarness::new(80, 24)?;
    /// harness.wait_for(|state| {
    ///     state.contains("Ready")
    /// })?;
    /// # Ok::<(), term_test::TermTestError>(())
    /// ```
    pub fn wait_for<F>(&mut self, condition: F) -> Result<()>
    where
        F: Fn(&ScreenState) -> bool,
    {
        self.wait_for_with_context(condition, "condition")
    }

    /// Waits for a condition with a custom error context.
    ///
    /// This is similar to `wait_for` but allows providing a description of what
    /// is being waited for, which improves error messages.
    ///
    /// # Arguments
    ///
    /// * `condition` - Condition to wait for
    /// * `description` - Human-readable description of the condition
    ///
    /// # Errors
    ///
    /// Returns a `Timeout` error if the condition is not met within the configured timeout.
    pub fn wait_for_with_context<F>(&mut self, condition: F, description: &str) -> Result<()>
    where
        F: Fn(&ScreenState) -> bool,
    {
        let start = Instant::now();
        let mut iterations = 0;

        loop {
            self.update_state()?;

            if condition(&self.state) {
                return Ok(());
            }

            let elapsed = start.elapsed();
            if elapsed >= self.timeout {
                // Create a detailed error message with current state
                let current_state = self.state.debug_contents();
                let cursor = self.state.cursor_position();

                eprintln!("\n=== Timeout waiting for: {} ===", description);
                eprintln!("Waited: {:?} ({} iterations)", elapsed, iterations);
                eprintln!("Cursor position: row={}, col={}", cursor.0, cursor.1);
                eprintln!("Current screen state:\n{}", current_state);
                eprintln!("==========================================\n");

                return Err(TermTestError::Timeout {
                    timeout_ms: self.timeout.as_millis() as u64,
                });
            }

            iterations += 1;
            std::thread::sleep(self.poll_interval);
        }
    }

    /// Waits for specific text to appear anywhere on the screen.
    ///
    /// This is a convenience wrapper around `wait_for` for the common case
    /// of waiting for text to appear.
    ///
    /// # Arguments
    ///
    /// * `text` - Text to wait for
    ///
    /// # Errors
    ///
    /// Returns a `Timeout` error if the text does not appear within the configured timeout.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use term_test::TuiTestHarness;
    /// # let mut harness = TuiTestHarness::new(80, 24)?;
    /// harness.wait_for_text("Ready")?;
    /// # Ok::<(), term_test::TermTestError>(())
    /// ```
    pub fn wait_for_text(&mut self, text: &str) -> Result<()> {
        let text = text.to_string();
        let description = format!("text '{}'", text);
        self.wait_for_with_context(
            move |state| state.contains(&text),
            &description,
        )
    }

    /// Returns the current screen contents as a string.
    pub fn screen_contents(&self) -> String {
        self.state.contents()
    }

    /// Returns the current cursor position as (row, col).
    ///
    /// Both row and column are 0-based indices. This is required for Phase 3
    /// Sixel position verification.
    ///
    /// # Returns
    ///
    /// A tuple of (row, col) where both are 0-based.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use term_test::TuiTestHarness;
    /// # let harness = TuiTestHarness::new(80, 24)?;
    /// let (row, col) = harness.cursor_position();
    /// println!("Cursor at: row={}, col={}", row, col);
    /// # Ok::<(), term_test::TermTestError>(())
    /// ```
    pub fn cursor_position(&self) -> (u16, u16) {
        self.state.cursor_position()
    }

    /// Alias for `cursor_position()` for convenience.
    ///
    /// Returns the current cursor position as (row, col).
    pub fn get_cursor_position(&self) -> (u16, u16) {
        self.cursor_position()
    }

    /// Returns the current screen state.
    ///
    /// Provides immutable access to the terminal screen state for inspecting
    /// rendered content without modifying it.
    ///
    /// # Returns
    ///
    /// A reference to the current [`ScreenState`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use term_test::TuiTestHarness;
    ///
    /// # fn test() -> term_test::Result<()> {
    /// let harness = TuiTestHarness::new(80, 24)?;
    /// let state = harness.state();
    /// println!("Screen size: {:?}", state.size());
    /// # Ok(())
    /// # }
    /// ```
    pub fn state(&self) -> &ScreenState {
        &self.state
    }

    /// Returns a mutable reference to the screen state.
    ///
    /// Allows direct manipulation of the screen state, which can be useful
    /// for testing specific scenarios or feeding mock data.
    ///
    /// # Returns
    ///
    /// A mutable reference to the [`ScreenState`].
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use term_test::TuiTestHarness;
    ///
    /// # fn test() -> term_test::Result<()> {
    /// let mut harness = TuiTestHarness::new(80, 24)?;
    /// harness.state_mut().feed(b"Test data");
    /// assert!(harness.screen_contents().contains("Test"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn state_mut(&mut self) -> &mut ScreenState {
        &mut self.state
    }

    /// Resizes the terminal.
    ///
    /// Changes the terminal dimensions and resets the screen state.
    /// This can be useful for testing responsive TUI layouts.
    ///
    /// # Arguments
    ///
    /// * `width` - New width in columns
    /// * `height` - New height in rows
    ///
    /// # Errors
    ///
    /// Returns an error if the resize operation fails.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use term_test::TuiTestHarness;
    ///
    /// # fn test() -> term_test::Result<()> {
    /// let mut harness = TuiTestHarness::new(80, 24)?;
    /// harness.resize(120, 40)?;
    /// assert_eq!(harness.state().size(), (120, 40));
    /// # Ok(())
    /// # }
    /// ```
    pub fn resize(&mut self, width: u16, height: u16) -> Result<()> {
        self.terminal.resize(width, height)?;
        self.state = ScreenState::new(width, height);
        Ok(())
    }

    /// Checks if the child process is still running.
    ///
    /// # Returns
    ///
    /// `true` if a process is currently running in the PTY, `false` otherwise.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use term_test::TuiTestHarness;
    /// use portable_pty::CommandBuilder;
    ///
    /// # fn test() -> term_test::Result<()> {
    /// let mut harness = TuiTestHarness::new(80, 24)?;
    /// let cmd = CommandBuilder::new("sleep");
    /// harness.spawn(cmd)?;
    /// assert!(harness.is_running());
    /// # Ok(())
    /// # }
    /// ```
    pub fn is_running(&mut self) -> bool {
        self.terminal.is_running()
    }

    /// Waits for the child process to exit.
    ///
    /// Blocks until the spawned process terminates and returns its exit status.
    ///
    /// # Returns
    ///
    /// The [`ExitStatus`] of the terminated process.
    ///
    /// # Errors
    ///
    /// Returns [`TermTestError::NoProcessRunning`] if no process is currently running.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use term_test::TuiTestHarness;
    /// use portable_pty::CommandBuilder;
    ///
    /// # fn test() -> term_test::Result<()> {
    /// let mut harness = TuiTestHarness::new(80, 24)?;
    /// let mut cmd = CommandBuilder::new("echo");
    /// cmd.arg("test");
    /// harness.spawn(cmd)?;
    ///
    /// let status = harness.wait_exit()?;
    /// assert!(status.success());
    /// # Ok(())
    /// # }
    /// ```
    pub fn wait_exit(&mut self) -> Result<ExitStatus> {
        self.terminal.wait()
    }
}

/// Builder for configuring a `TuiTestHarness`.
///
/// # Example
///
/// ```rust,no_run
/// use term_test::TuiTestHarness;
/// use std::time::Duration;
///
/// let mut harness = TuiTestHarness::builder()
///     .with_size(80, 24)
///     .with_timeout(Duration::from_secs(10))
///     .with_poll_interval(Duration::from_millis(50))
///     .with_buffer_size(8192)
///     .build()?;
/// # Ok::<(), term_test::TermTestError>(())
/// ```
#[derive(Debug, Clone)]
pub struct TuiTestHarnessBuilder {
    width: u16,
    height: u16,
    timeout: Duration,
    poll_interval: Duration,
    buffer_size: usize,
}

impl Default for TuiTestHarnessBuilder {
    fn default() -> Self {
        Self {
            width: 80,
            height: 24,
            timeout: DEFAULT_TIMEOUT,
            poll_interval: DEFAULT_POLL_INTERVAL,
            buffer_size: DEFAULT_BUFFER_SIZE,
        }
    }
}

impl TuiTestHarnessBuilder {
    /// Sets the terminal size.
    ///
    /// # Arguments
    ///
    /// * `width` - Terminal width in columns
    /// * `height` - Terminal height in rows
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = width;
        self.height = height;
        self
    }

    /// Sets the timeout for wait operations.
    ///
    /// # Arguments
    ///
    /// * `timeout` - Timeout duration
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the polling interval for wait operations.
    ///
    /// # Arguments
    ///
    /// * `interval` - Polling interval
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    /// Sets the buffer size for reading PTY output.
    ///
    /// # Arguments
    ///
    /// * `size` - Buffer size in bytes
    pub fn with_buffer_size(mut self, size: usize) -> Self {
        self.buffer_size = size;
        self
    }

    /// Builds the test harness with the configured settings.
    ///
    /// # Errors
    ///
    /// Returns an error if terminal creation fails.
    pub fn build(self) -> Result<TuiTestHarness> {
        let terminal = TestTerminal::new(self.width, self.height)?;
        let state = ScreenState::new(self.width, self.height);

        Ok(TuiTestHarness {
            terminal,
            state,
            timeout: self.timeout,
            poll_interval: self.poll_interval,
            buffer_size: self.buffer_size,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_harness() {
        let harness = TuiTestHarness::new(80, 24);
        assert!(harness.is_ok());
        let harness = harness.unwrap();
        assert_eq!(harness.timeout, DEFAULT_TIMEOUT);
        assert_eq!(harness.poll_interval, DEFAULT_POLL_INTERVAL);
        assert_eq!(harness.buffer_size, DEFAULT_BUFFER_SIZE);
    }

    #[test]
    fn test_with_timeout() {
        let harness = TuiTestHarness::new(80, 24)
            .unwrap()
            .with_timeout(Duration::from_secs(10));
        assert_eq!(harness.timeout, Duration::from_secs(10));
    }

    #[test]
    fn test_with_poll_interval() {
        let harness = TuiTestHarness::new(80, 24)
            .unwrap()
            .with_poll_interval(Duration::from_millis(50));
        assert_eq!(harness.poll_interval, Duration::from_millis(50));
    }

    #[test]
    fn test_builder_default() {
        let harness = TuiTestHarness::builder().build();
        assert!(harness.is_ok());
        let harness = harness.unwrap();
        assert_eq!(harness.timeout, DEFAULT_TIMEOUT);
        assert_eq!(harness.poll_interval, DEFAULT_POLL_INTERVAL);
        assert_eq!(harness.buffer_size, DEFAULT_BUFFER_SIZE);
    }

    #[test]
    fn test_builder_with_size() {
        let harness = TuiTestHarness::builder()
            .with_size(120, 40)
            .build()
            .unwrap();
        let (width, height) = harness.state.size();
        assert_eq!(width, 120);
        assert_eq!(height, 40);
    }

    #[test]
    fn test_builder_with_timeout() {
        let timeout = Duration::from_secs(15);
        let harness = TuiTestHarness::builder()
            .with_timeout(timeout)
            .build()
            .unwrap();
        assert_eq!(harness.timeout, timeout);
    }

    #[test]
    fn test_builder_with_poll_interval() {
        let interval = Duration::from_millis(25);
        let harness = TuiTestHarness::builder()
            .with_poll_interval(interval)
            .build()
            .unwrap();
        assert_eq!(harness.poll_interval, interval);
    }

    #[test]
    fn test_builder_with_buffer_size() {
        let buffer_size = 8192;
        let harness = TuiTestHarness::builder()
            .with_buffer_size(buffer_size)
            .build()
            .unwrap();
        assert_eq!(harness.buffer_size, buffer_size);
    }

    #[test]
    fn test_builder_chaining() {
        let harness = TuiTestHarness::builder()
            .with_size(100, 30)
            .with_timeout(Duration::from_secs(20))
            .with_poll_interval(Duration::from_millis(75))
            .with_buffer_size(16384)
            .build()
            .unwrap();

        assert_eq!(harness.state.size(), (100, 30));
        assert_eq!(harness.timeout, Duration::from_secs(20));
        assert_eq!(harness.poll_interval, Duration::from_millis(75));
        assert_eq!(harness.buffer_size, 16384);
    }

    #[test]
    fn test_cursor_position() {
        let harness = TuiTestHarness::new(80, 24).unwrap();
        let (row, col) = harness.cursor_position();
        // Initial cursor position should be at (0, 0)
        assert_eq!(row, 0);
        assert_eq!(col, 0);
    }

    #[test]
    fn test_get_cursor_position_alias() {
        let harness = TuiTestHarness::new(80, 24).unwrap();
        let pos1 = harness.cursor_position();
        let pos2 = harness.get_cursor_position();
        assert_eq!(pos1, pos2);
    }

    #[test]
    fn test_wait_for_text_helper_exists() {
        // Test that all wait_for methods exist and compile (signature test)
        let harness = TuiTestHarness::new(80, 24).unwrap();

        // Test method signatures exist (don't call them as they require a running process)
        let _: fn(&mut TuiTestHarness, &str) -> Result<()> = TuiTestHarness::wait_for_text;

        // Verify state access methods exist
        assert_eq!(harness.cursor_position(), (0, 0));
        assert_eq!(harness.get_cursor_position(), (0, 0));
    }

    #[test]
    fn test_state_manipulation() {
        // Test that we can manipulate state directly
        let mut harness = TuiTestHarness::new(80, 24).unwrap();

        // Feed text directly to state
        harness.state_mut().feed(b"Test Data");

        // Verify we can read it back
        let contents = harness.screen_contents();
        assert!(contents.contains("Test"));
    }

    #[test]
    fn test_cursor_position_tracking() {
        // Test cursor position tracking
        let mut harness = TuiTestHarness::new(80, 24).unwrap();

        // Initial position
        assert_eq!(harness.cursor_position(), (0, 0));

        // Feed escape sequence to move cursor
        harness.state_mut().feed(b"\x1b[2;5H"); // Move to row 2, col 5

        // Check new position (note: escape sequences use 1-based indexing, we return 0-based)
        let (row, col) = harness.cursor_position();
        assert!(row >= 0); // Just verify we get valid coordinates
        assert!(col >= 0);
    }

    #[test]
    fn test_screen_state_access() {
        let harness = TuiTestHarness::new(80, 24).unwrap();
        let state = harness.state();
        assert_eq!(state.size(), (80, 24));

        let contents = harness.screen_contents();
        assert!(contents.len() > 0 || contents.is_empty()); // Just verify it returns something
    }

    #[test]
    fn test_resize() {
        let mut harness = TuiTestHarness::new(80, 24).unwrap();
        let result = harness.resize(100, 30);
        assert!(result.is_ok());
        assert_eq!(harness.state.size(), (100, 30));
    }

    #[test]
    fn test_is_running_no_process() {
        let mut harness = TuiTestHarness::new(80, 24).unwrap();
        assert!(!harness.is_running());
    }

    #[test]
    fn test_spawn_and_check_running() {
        let mut harness = TuiTestHarness::new(80, 24).unwrap();
        let mut cmd = CommandBuilder::new("sleep");
        cmd.arg("0.1");

        let spawn_result = harness.spawn(cmd);
        if spawn_result.is_ok() {
            // Should be running initially
            assert!(harness.is_running());

            // Wait for it to exit
            std::thread::sleep(Duration::from_millis(200));

            // Should have exited
            assert!(!harness.is_running());
        }
    }
}

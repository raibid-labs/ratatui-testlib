//! Scarab-specific IPC and shared-memory helpers.
//!
//! This module provides Scarab-tailored wrappers around the generic [`ipc`](crate::ipc) module,
//! preconfigured for Scarab's default paths and protocol.
//!
//! # Overview
//!
//! Scarab uses a split architecture (daemon + GPU client) where:
//! - **scarab-daemon** manages the PTY, parses terminal state, and exposes it via shared memory
//! - **scarab** (GPU client) renders the UI by reading from shared memory
//!
//! This module provides the glue to test Scarab without reimplementing IPC/shared-memory plumbing.
//!
//! # Quick Start
//!
//! ## Enable the feature
//!
//! ```toml
//! [dependencies]
//! ratatui-testlib = { version = "0.3", features = ["scarab"] }
//! ```
//!
//! ## Environment Variable
//!
//! Enable Scarab testing mode by setting:
//!
//! ```bash
//! export SCARAB_TEST_RTL=1
//! ```
//!
//! ## Basic Test Example
//!
//! ```rust,no_run
//! # #[cfg(feature = "scarab")]
//! # {
//! use std::time::Duration;
//! use ratatui_testlib::scarab::ScarabTestHarness;
//!
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! // Connect to running scarab-daemon using default paths
//! let mut harness = ScarabTestHarness::connect()?;
//!
//! // Send input via IPC
//! harness.send_input("echo hello\n")?;
//!
//! // Wait for output in shared memory grid
//! harness.wait_for_text("hello", Duration::from_secs(5))?;
//!
//! // Assert grid contents
//! let grid = harness.grid_contents()?;
//! assert!(grid.contains("hello"));
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! # Default Configuration
//!
//! Scarab uses these default paths:
//! - Socket: `/tmp/scarab-daemon.sock`
//! - Shared memory: `/scarab_shm_v1`
//! - Image buffer: `/scarab_img_v1`
//! - Magic number: `0x5343_5241` ("SCRA")
//!
//! # Custom Configuration
//!
//! ```rust,no_run
//! # #[cfg(feature = "scarab")]
//! # {
//! use std::time::Duration;
//! use ratatui_testlib::scarab::{ScarabTestHarness, ScarabConfig};
//!
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! let config = ScarabConfig::builder()
//!     .socket_path("/tmp/test-scarab.sock")
//!     .shm_path("/test_scarab_shm")
//!     .dimensions(120, 40)
//!     .build();
//!
//! let mut harness = ScarabTestHarness::with_config(config)?;
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! # Testing Patterns
//!
//! ## Pattern 1: Wait for Shell Prompt
//!
//! ```rust,no_run
//! # #[cfg(feature = "scarab")]
//! # {
//! use std::time::Duration;
//! use ratatui_testlib::scarab::ScarabTestHarness;
//!
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! let mut harness = ScarabTestHarness::connect()?;
//!
//! // Wait for shell prompt
//! harness.wait_for_prompt(Duration::from_secs(5))?;
//!
//! // Send command
//! harness.send_input("ls -la\n")?;
//!
//! // Wait for output and next prompt
//! harness.wait_for_text("total", Duration::from_secs(2))?;
//! # Ok(())
//! # }
//! # }
//! ```
//!
//! ## Pattern 2: Test Escape Sequences
//!
//! ```rust,no_run
//! # #[cfg(feature = "scarab")]
//! # {
//! use std::time::Duration;
//! use ratatui_testlib::scarab::ScarabTestHarness;
//!
//! # fn test() -> Result<(), Box<dyn std::error::Error>> {
//! let mut harness = ScarabTestHarness::connect()?;
//!
//! // Send cursor movement
//! harness.send_input("\x1b[A")?; // Up arrow
//!
//! // Verify cursor position changed
//! let (row, _col) = harness.cursor_position()?;
//! println!("Cursor at row: {}", row);
//! # Ok(())
//! # }
//! # }
//! ```

use std::{
    path::PathBuf,
    time::Duration,
};

use crate::ipc::{DaemonIpcClient, DaemonSharedMemory, IpcError, IpcResult};

// Scarab-specific defaults
const SCARAB_SOCKET_PATH: &str = "/tmp/scarab-daemon.sock";
const SCARAB_SHM_PATH: &str = "/scarab_shm_v1";
const SCARAB_IMAGE_SHM_PATH: &str = "/scarab_img_v1";
const SCARAB_MAGIC: u32 = 0x5343_5241; // "SCRA"
const SCARAB_VERSION: u32 = 1;

/// Scarab-specific configuration.
///
/// Preconfigured with Scarab's default paths and protocol settings.
#[derive(Debug, Clone)]
pub struct ScarabConfig {
    /// Path to the Unix socket for IPC.
    pub socket_path: PathBuf,

    /// Path to the shared memory segment for terminal state.
    pub shm_path: String,

    /// Path to the shared memory segment for image buffer.
    pub image_shm_path: Option<String>,

    /// Terminal dimensions (cols, rows).
    pub dimensions: Option<(u16, u16)>,

    /// Connection timeout.
    pub connect_timeout: Duration,

    /// Default timeout for wait operations.
    pub default_timeout: Duration,

    /// Prompt patterns to detect (e.g., "$", "#", ">").
    pub prompt_patterns: Vec<String>,
}

impl Default for ScarabConfig {
    fn default() -> Self {
        Self {
            socket_path: PathBuf::from(SCARAB_SOCKET_PATH),
            shm_path: SCARAB_SHM_PATH.to_string(),
            image_shm_path: Some(SCARAB_IMAGE_SHM_PATH.to_string()),
            dimensions: Some((80, 24)),
            connect_timeout: Duration::from_secs(5),
            default_timeout: Duration::from_secs(10),
            prompt_patterns: vec![
                "$ ".to_string(),
                "# ".to_string(),
                "> ".to_string(),
            ],
        }
    }
}

impl ScarabConfig {
    /// Create a new configuration builder.
    pub fn builder() -> ScarabConfigBuilder {
        ScarabConfigBuilder::default()
    }

}

/// Builder for ScarabConfig.
#[derive(Debug, Default)]
pub struct ScarabConfigBuilder {
    config: ScarabConfig,
}

impl ScarabConfigBuilder {
    /// Set the Unix socket path.
    pub fn socket_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.socket_path = path.into();
        self
    }

    /// Set the shared memory path.
    pub fn shm_path(mut self, path: impl Into<String>) -> Self {
        self.config.shm_path = path.into();
        self
    }

    /// Set the image shared memory path.
    pub fn image_shm_path(mut self, path: impl Into<String>) -> Self {
        self.config.image_shm_path = Some(path.into());
        self
    }

    /// Set terminal dimensions.
    pub fn dimensions(mut self, cols: u16, rows: u16) -> Self {
        self.config.dimensions = Some((cols, rows));
        self
    }

    /// Set connection timeout.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.config.connect_timeout = timeout;
        self
    }

    /// Set default wait timeout.
    pub fn default_timeout(mut self, timeout: Duration) -> Self {
        self.config.default_timeout = timeout;
        self
    }

    /// Set prompt patterns.
    pub fn prompt_patterns(mut self, patterns: Vec<String>) -> Self {
        self.config.prompt_patterns = patterns;
        self
    }

    /// Add a prompt pattern.
    pub fn add_prompt_pattern(mut self, pattern: impl Into<String>) -> Self {
        self.config.prompt_patterns.push(pattern.into());
        self
    }

    /// Build the configuration.
    pub fn build(self) -> ScarabConfig {
        self.config
    }
}

/// Scarab-specific shared memory reader.
///
/// Wraps [`DaemonSharedMemory`] with Scarab's magic number and version validation.
#[cfg(target_family = "unix")]
pub struct ScarabSharedMemory {
    inner: DaemonSharedMemory,
}

#[cfg(target_family = "unix")]
impl std::fmt::Debug for ScarabSharedMemory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ScarabSharedMemory")
            .field("inner", &self.inner)
            .finish()
    }
}

#[cfg(target_family = "unix")]
impl ScarabSharedMemory {
    /// Open Scarab shared memory with protocol validation.
    pub fn open(shm_path: &str) -> IpcResult<Self> {
        let inner = DaemonSharedMemory::open_with_validation(
            shm_path,
            SCARAB_MAGIC,
            SCARAB_VERSION,
        )?;
        Ok(Self { inner })
    }

    /// Refresh the header from shared memory.
    pub fn refresh(&mut self) -> IpcResult<()> {
        self.inner.refresh()
    }

    /// Get the terminal dimensions (cols, rows).
    pub fn dimensions(&self) -> (u16, u16) {
        self.inner.dimensions()
    }

    /// Get the cursor position (row, col).
    pub fn cursor_position(&self) -> (u16, u16) {
        self.inner.cursor_position()
    }

    /// Get the sequence number for change detection.
    pub fn sequence_number(&self) -> u32 {
        self.inner.sequence_number()
    }

    /// Read the terminal grid as a string.
    pub fn grid_contents(&self) -> IpcResult<String> {
        self.inner.grid_contents()
    }

    /// Check if the grid contains the given text.
    pub fn contains(&self, text: &str) -> IpcResult<bool> {
        self.inner.contains(text)
    }

    /// Get a specific cell character at (row, col).
    pub fn cell_at(&self, row: u16, col: u16) -> IpcResult<char> {
        self.inner.cell_at(row, col)
    }
}

/// Scarab test harness for integration testing.
///
/// Combines IPC communication and shared memory reading into a single
/// ergonomic API tailored for Scarab testing.
///
/// # Example
///
/// ```rust,no_run
/// # #[cfg(feature = "scarab")]
/// # {
/// use std::time::Duration;
/// use ratatui_testlib::scarab::ScarabTestHarness;
///
/// # fn test() -> Result<(), Box<dyn std::error::Error>> {
/// let mut harness = ScarabTestHarness::connect()?;
///
/// // Send a command
/// harness.send_input("echo 'Hello, Scarab!'\n")?;
///
/// // Wait for output
/// harness.wait_for_text("Hello, Scarab!", Duration::from_secs(5))?;
/// # Ok(())
/// # }
/// # }
/// ```
#[cfg(target_family = "unix")]
#[derive(Debug)]
pub struct ScarabTestHarness {
    ipc: DaemonIpcClient,
    shm: ScarabSharedMemory,
    config: ScarabConfig,
}

#[cfg(target_family = "unix")]
impl ScarabTestHarness {
    /// Check if Scarab testing is enabled via environment variable.
    pub fn is_enabled() -> bool {
        std::env::var("SCARAB_TEST_RTL").is_ok()
    }

    /// Connect to a running Scarab daemon using default configuration.
    ///
    /// Requires `SCARAB_TEST_RTL=1` environment variable to be set.
    pub fn connect() -> IpcResult<Self> {
        if !Self::is_enabled() {
            return Err(IpcError::TestingDisabled);
        }
        Self::with_config(ScarabConfig::default())
    }

    /// Create a harness with custom configuration.
    pub fn with_config(config: ScarabConfig) -> IpcResult<Self> {
        // Connect to IPC socket
        let ipc = DaemonIpcClient::connect(&config.socket_path)?;

        // Open shared memory with Scarab-specific validation
        let shm = ScarabSharedMemory::open(&config.shm_path)?;

        Ok(Self { ipc, shm, config })
    }

    /// Send input text to the PTY via IPC.
    pub fn send_input(&mut self, text: &str) -> IpcResult<()> {
        self.ipc.send_text(text)
    }

    /// Send raw bytes to the PTY via IPC.
    pub fn send_bytes(&mut self, bytes: &[u8]) -> IpcResult<()> {
        self.ipc.send_input(bytes)
    }

    /// Resize the terminal.
    pub fn resize(&mut self, cols: u16, rows: u16) -> IpcResult<()> {
        self.ipc.resize(cols, rows)
    }

    /// Request a state refresh from the daemon.
    pub fn refresh(&mut self) -> IpcResult<()> {
        self.ipc.refresh()?;
        self.shm.refresh()
    }

    /// Get the current grid contents as a string.
    pub fn grid_contents(&self) -> IpcResult<String> {
        self.shm.grid_contents()
    }

    /// Get the current cursor position (row, col).
    pub fn cursor_position(&self) -> IpcResult<(u16, u16)> {
        Ok(self.shm.cursor_position())
    }

    /// Get the terminal dimensions (cols, rows).
    pub fn dimensions(&self) -> (u16, u16) {
        self.shm.dimensions()
    }

    /// Check if the grid contains the given text.
    pub fn contains(&self, text: &str) -> IpcResult<bool> {
        self.shm.contains(text)
    }

    /// Wait until the grid contains the specified text.
    pub fn wait_for_text(&mut self, text: &str, timeout: Duration) -> IpcResult<()> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(50);

        loop {
            self.shm.refresh()?;

            if self.shm.contains(text)? {
                return Ok(());
            }

            if start.elapsed() >= timeout {
                return Err(IpcError::Timeout(timeout));
            }

            std::thread::sleep(poll_interval);
        }
    }

    /// Wait until the grid does NOT contain the specified text.
    pub fn wait_for_text_absent(&mut self, text: &str, timeout: Duration) -> IpcResult<()> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(50);

        loop {
            self.shm.refresh()?;

            if !self.shm.contains(text)? {
                return Ok(());
            }

            if start.elapsed() >= timeout {
                return Err(IpcError::Timeout(timeout));
            }

            std::thread::sleep(poll_interval);
        }
    }

    /// Wait for a shell prompt to appear.
    ///
    /// Uses the configured prompt patterns (default: `$`, `#`, `>`).
    pub fn wait_for_prompt(&mut self, timeout: Duration) -> IpcResult<()> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(50);
        let patterns = self.config.prompt_patterns.clone();

        loop {
            self.shm.refresh()?;
            let grid = self.shm.grid_contents()?;

            for pattern in &patterns {
                if grid.contains(pattern) {
                    return Ok(());
                }
            }

            if start.elapsed() >= timeout {
                return Err(IpcError::Timeout(timeout));
            }

            std::thread::sleep(poll_interval);
        }
    }

    /// Wait for a sequence of text strings to appear in order.
    pub fn wait_for_sequence(&mut self, texts: &[&str], timeout: Duration) -> IpcResult<()> {
        let start = std::time::Instant::now();

        for text in texts {
            let remaining = timeout.saturating_sub(start.elapsed());
            if remaining.is_zero() {
                return Err(IpcError::Timeout(timeout));
            }
            self.wait_for_text(text, remaining)?;
        }

        Ok(())
    }

    /// Wait for the sequence number to change, indicating a state update.
    pub fn wait_for_update(&mut self, timeout: Duration) -> IpcResult<()> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(10);
        let initial_seq = self.shm.sequence_number();

        loop {
            self.shm.refresh()?;

            if self.shm.sequence_number() != initial_seq {
                return Ok(());
            }

            if start.elapsed() >= timeout {
                return Err(IpcError::Timeout(timeout));
            }

            std::thread::sleep(poll_interval);
        }
    }

    /// Assert that the grid contains the expected text.
    pub fn assert_contains(&self, text: &str) -> IpcResult<()> {
        if self.shm.contains(text)? {
            Ok(())
        } else {
            Err(IpcError::InvalidData(format!(
                "Expected grid to contain '{}', but it didn't.\nGrid:\n{}",
                text,
                self.shm.grid_contents().unwrap_or_default()
            )))
        }
    }

    /// Get the default timeout from configuration.
    pub fn default_timeout(&self) -> Duration {
        self.config.default_timeout
    }

    /// Get a reference to the underlying shared memory reader.
    pub fn shared_memory(&self) -> &ScarabSharedMemory {
        &self.shm
    }
}

/// Extension trait for integrating Scarab testing with TuiTestHarness.
pub trait ScarabTestExt {
    /// Connect to a Scarab daemon for testing.
    fn connect_scarab(&self) -> IpcResult<ScarabTestHarness>;

    /// Connect to a Scarab daemon with custom configuration.
    fn connect_scarab_with_config(&self, config: ScarabConfig) -> IpcResult<ScarabTestHarness>;

    /// Check if Scarab testing mode is enabled.
    fn scarab_enabled(&self) -> bool;
}

impl ScarabTestExt for crate::TuiTestHarness {
    fn connect_scarab(&self) -> IpcResult<ScarabTestHarness> {
        ScarabTestHarness::connect()
    }

    fn connect_scarab_with_config(&self, config: ScarabConfig) -> IpcResult<ScarabTestHarness> {
        ScarabTestHarness::with_config(config)
    }

    fn scarab_enabled(&self) -> bool {
        ScarabTestHarness::is_enabled()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = ScarabConfig::default();

        assert_eq!(config.socket_path, PathBuf::from("/tmp/scarab-daemon.sock"));
        assert_eq!(config.shm_path, "/scarab_shm_v1");
        assert_eq!(config.image_shm_path, Some("/scarab_img_v1".to_string()));
        assert_eq!(config.dimensions, Some((80, 24)));
        assert!(!config.prompt_patterns.is_empty());
    }

    #[test]
    fn test_config_builder() {
        let config = ScarabConfig::builder()
            .socket_path("/custom/socket.sock")
            .shm_path("/custom_shm")
            .dimensions(120, 40)
            .prompt_patterns(vec![">>> ".to_string()])
            .build();

        assert_eq!(config.socket_path, PathBuf::from("/custom/socket.sock"));
        assert_eq!(config.shm_path, "/custom_shm");
        assert_eq!(config.dimensions, Some((120, 40)));
        assert_eq!(config.prompt_patterns, vec![">>> ".to_string()]);
    }

    #[test]
    fn test_add_prompt_pattern() {
        let config = ScarabConfig::builder()
            .add_prompt_pattern(">>> ")
            .add_prompt_pattern("... ")
            .build();

        // Default patterns plus two new ones
        assert!(config.prompt_patterns.len() >= 2);
        assert!(config.prompt_patterns.contains(&">>> ".to_string()));
        assert!(config.prompt_patterns.contains(&"... ".to_string()));
    }

    #[test]
    fn test_is_enabled_without_env() {
        std::env::remove_var("SCARAB_TEST_RTL");
        assert!(!ScarabTestHarness::is_enabled());
    }

    #[test]
    fn test_is_enabled_with_env() {
        std::env::set_var("SCARAB_TEST_RTL", "1");
        assert!(ScarabTestHarness::is_enabled());
        std::env::remove_var("SCARAB_TEST_RTL");
    }

    #[test]
    fn test_scarab_magic_constants() {
        assert_eq!(SCARAB_MAGIC, 0x5343_5241);
        assert_eq!(SCARAB_VERSION, 1);
    }
}

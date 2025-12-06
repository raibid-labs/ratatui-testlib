# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0] - 2025-12-06

### Added
- Documentation cleanup and versioning system (#34)
  - Versioned documentation structure in `docs/versions/vNEXT`
  - Documentation organization guide (`docs/STRUCTURE.md`)
  - Documentation validation script (`scripts/check-docs.sh`)
  - CI integration for documentation checks
- Release pipeline and branch protection documentation (#35)
  - Comprehensive release process guide (`docs/RELEASE.md`)
  - Branch protection configuration guide (`docs/BRANCH_PROTECTION.md`)
  - CODEOWNERS file for required reviewers
- Roadmap scaffolding and future features (#36)
  - Detailed roadmap in `docs/versions/vNEXT/ROADMAP.md`
  - Project cookbooks directory with Scarab file manager patterns
  - Examples gallery documentation (`examples/README.md`)
  - Ratatui version compatibility testing workflow
  - Enhanced release workflow with artifact documentation

### Changed
- Removed AI-generated implementation reports and conversation logs
- Updated README to reference versioned documentation
- Enhanced release workflow to document all artifacts (crate, docs, examples)

## [0.1.0] - Initial Release

### Added
- Initial implementation of ratatui-testlib TUI testing library
- PTY-based terminal emulation with `portable-pty`
- VT100/ANSI escape sequence parsing with `vtparse` and `termwiz`
- Sixel graphics position tracking and verification
- Bevy ECS integration for testing Bevy TUI applications
- Async/await support with Tokio
- Snapshot testing integration with `insta`
- Comprehensive test harness API (`TuiTestHarness`)
- Screen state inspection (`ScreenState`, `Cell`, `SixelRegion`)
- Event simulation (keyboard input, wait conditions)
- Examples for common testing scenarios
- Full documentation and API reference

### Features
- `async-tokio`: Tokio async runtime support
- `bevy`: Bevy ECS integration
- `bevy-ratatui`: bevy_ratatui plugin support
- `ratatui-helpers`: Ratatui-specific test helpers
- `sixel`: Sixel graphics position tracking
- `snapshot-insta`: Snapshot testing with insta
- `mvp`: Bundle of all MVP features

### Public API
- `TuiTestHarness`: Main test harness for PTY-based testing
- `ScreenState`: Terminal screen state with VT100 parsing
- `ScreenState::new(width, height)`: Create new screen state
- `ScreenState::feed(data)`: Process terminal escape sequences
- `ScreenState::get_cell(row, col)`: Access individual cells
- `ScreenState::contents()`: Get screen contents as string
- `ScreenState::sixel_regions()`: Get Sixel graphics regions
- `Cell`: Terminal cell with character and attributes
- `SixelRegion`: Sixel graphics position and data
- `TestTerminal`: PTY management wrapper
- `BevyTuiTestHarness`: Bevy-specific test harness

### Addresses
- Issue #1: TUI Integration Testing Framework Requirements - Fully implemented
- Issue #7: Add public API for headless/stream-based parsing - Implemented via `ScreenState::feed()`
- Issue #8: Expose Screen/Grid state for verification - Implemented via `get_cell()` and public `Cell` struct

[Unreleased]: https://github.com/raibid-labs/ratatui-testlib/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/raibid-labs/ratatui-testlib/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/raibid-labs/ratatui-testlib/releases/tag/v0.1.0

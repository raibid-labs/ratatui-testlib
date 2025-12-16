# Contributing to terminal-testlib

Thank you for your interest in contributing to `terminal-testlib`! We welcome contributions from the community to help make TUI testing easier and more robust.

## Getting Started

1.  **Fork the repository** on GitHub.
2.  **Clone your fork** locally.
3.  **Install Rust**: Ensure you have the latest stable Rust toolchain installed via [rustup](https://rustup.rs/).

## Development Workflow

### Running Tests

We have a comprehensive test suite. Please ensure all tests pass before submitting a PR.

```bash
# Run all unit and integration tests
cargo test --all-features

# Run specific test
cargo test test_name
```

**Note on Golden File Tests**: Some visual regression tests (`tests/golden_files.rs`) manipulate environment variables and might need to be run serially:
```bash
cargo test --test golden_files -- --test-threads=1
```

### Updating Golden Files

If you make changes that intentionally affect the visual output of terminal tests, you may need to update the "golden" reference files:

```bash
UPDATE_GOLDENS=1 cargo test
```

### Code Style

We use `rustfmt` and `clippy` to enforce code style and quality.

```bash
# Format code
cargo fmt

# Lint code
cargo clippy --all-features -- -D warnings
```

## Project Structure

*   `src/pty.rs`: Low-level PTY management wrapper around `portable-pty`.
*   `src/screen.rs`: Terminal emulation state (parsing VT100/ANSI sequences).
*   `src/harness.rs`: Main `TuiTestHarness` implementation.
*   `src/async_harness.rs`: Tokio-based async wrapper.
*   `src/sixel.rs` / `src/graphics.rs`: Graphics protocol support.
*   `src/bevy/`: Bevy ECS integration.
*   `examples/`: Usage examples.

## Reporting Issues

If you find a bug or have a feature request, please open an issue on GitHub. Provide as much detail as possible, including:
*   Steps to reproduce the issue.
*   Expected vs. actual behavior.
*   Environment details (OS, terminal emulator if relevant).

## License

By contributing, you agree that your contributions will be licensed under the project's MIT License.

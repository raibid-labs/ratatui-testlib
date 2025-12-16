# terminal-testlib Examples

This directory contains examples demonstrating various features and patterns of terminal-testlib.

## Quick Start

Run any example with:
```bash
cargo run --example <example_name> --all-features
```

For example:
```bash
cargo run --example basic_test --all-features
```

## Example Index

### Beginner Examples

| Example | Description | Features |
|---------|-------------|----------|
| [basic_test.rs](basic_test.rs) | Simple PTY harness usage | PTY, Wait, Basic assertions |
| [harness_demo.rs](harness_demo.rs) | Test harness fundamentals | Setup, Spawn, Cleanup |
| [wait_demo.rs](wait_demo.rs) | Wait condition patterns | Timeouts, Conditions, Polling |
| [keyboard_demo.rs](keyboard_demo.rs) | Keyboard input simulation | Key events, Text input |
| [mouse_demo.rs](mouse_demo.rs) | Mouse event simulation | Click, Drag, Position |

### Intermediate Examples

| Example | Description | Features |
|---------|-------------|----------|
| [navigation_demo.rs](navigation_demo.rs) | Navigation patterns | Vim keys, Modal dialogs |
| [grid_verification.rs](grid_verification.rs) | Grid position testing | Cell assertions, Regions |
| [snapshot_test.rs](snapshot_test.rs) | Snapshot testing | Insta integration, Golden files |
| [position_test.rs](position_test.rs) | Position assertions | Cursor, Text location |
| [async_test.rs](async_test.rs) | Async testing patterns | Tokio, Async harness |

### Advanced Examples

| Example | Description | Features |
|---------|-------------|----------|
| [sixel_test.rs](sixel_test.rs) | Graphics protocol testing | Sixel detection, Position tracking |
| [bevy_test.rs](bevy_test.rs) | Bevy ECS integration | World access, Component queries |
| [headless_bevy.rs](headless_bevy.rs) | Headless Bevy testing | CI mode, No display required |
| [bevy_component_snapshot.rs](bevy_component_snapshot.rs) | Bevy snapshot testing | Component serialization |
| [headless_ci_example.rs](headless_ci_example.rs) | Full CI integration | Headless mode, All features |

### Specialized Examples

| Example | Description | Features |
|---------|-------------|----------|
| [graphics_detection.rs](graphics_detection.rs) | Graphics protocol detection | Sixel, iTerm2, Kitty |
| [terminal_profiles_demo.rs](terminal_profiles_demo.rs) | Terminal emulator testing | Profile selection, Capabilities |
| [stream_parsing.rs](stream_parsing.rs) | ANSI sequence parsing | Escape codes, Control sequences |
| [timing_demo.rs](timing_demo.rs) | Timing and performance | Render timing, Benchmarks |
| [benchmark_test.rs](benchmark_test.rs) | Performance testing | Criterion integration |

## Golden Snapshots

Some examples include golden snapshot tests that verify output against known-good baselines.

### Snapshot Location

Snapshots are stored in:
- Test snapshots: `tests/snapshots/`
- Example snapshots: (planned for `examples/snapshots/`)

### Running Snapshot Tests

```bash
# Run all tests including snapshots
cargo test --all-features

# Review and update snapshots
cargo insta review
```

### Creating New Snapshots

When adding a new example with snapshots:

1. Use the `insta` crate:
```rust
use insta::assert_snapshot;

#[test]
fn test_my_feature() {
    let output = harness.screen_text();
    assert_snapshot!(output);
}
```

2. Run test to create initial snapshot:
```bash
cargo test test_my_feature
```

3. Review snapshot:
```bash
cargo insta review
```

4. Commit the `.snap` file with your code

### Snapshot Best Practices

- ✅ Snapshot stable output (avoid timestamps, random data)
- ✅ Use descriptive snapshot names
- ✅ Review snapshots in PRs
- ✅ Update snapshots when behavior intentionally changes
- ❌ Don't commit snapshots with sensitive data
- ❌ Don't snapshot highly variable output

## Feature Flags

Examples may require specific feature flags:

```bash
# Async examples
cargo run --example async_test --features async-tokio

# Bevy examples
cargo run --example bevy_test --features bevy

# Sixel examples
cargo run --example sixel_test --features sixel

# Headless examples
cargo run --example headless_bevy --features bevy,headless

# All features
cargo run --example headless_ci_example --all-features
```

## Example Categories

### By Use Case

**File Manager/Browser**
- navigation_demo.rs - File list navigation
- grid_verification.rs - Position verification

**Real-time Applications**
- async_test.rs - Async event handling
- timing_demo.rs - Performance monitoring

**Game Development**
- bevy_test.rs - ECS integration
- bevy_component_snapshot.rs - State snapshots

**Graphics Applications**
- sixel_test.rs - Image rendering
- graphics_detection.rs - Protocol support

**CI/CD Testing**
- headless_ci_example.rs - Automated testing
- headless_bevy.rs - Headless Bevy

### By Difficulty

**Beginner**: basic_test, harness_demo, wait_demo, keyboard_demo

**Intermediate**: navigation_demo, snapshot_test, async_test, grid_verification

**Advanced**: sixel_test, bevy_test, headless_ci_example, stream_parsing

## Running All Examples

Test that all examples compile:
```bash
cargo build --examples --all-features
```

Run all examples (some may require interaction):
```bash
for example in examples/*.rs; do
    name=$(basename "$example" .rs)
    echo "Running $name..."
    cargo run --example "$name" --all-features || true
done
```

## Contributing Examples

When adding a new example:

1. **Choose a clear name**: `feature_name_demo.rs` or `feature_test.rs`
2. **Add documentation**: Include module-level docs explaining the example
3. **Keep it focused**: Demonstrate one feature or pattern clearly
4. **Add to index**: Update this README with your example
5. **Test it works**: Ensure it runs with `--all-features`
6. **Consider snapshots**: Add golden snapshots if appropriate

Example template:
```rust
//! Brief description of what this example demonstrates
//!
//! # Features
//! - Feature 1
//! - Feature 2
//!
//! # Usage
//! ```bash
//! cargo run --example my_example --features needed-features
//! ```

use terminal_testlib::TuiTestHarness;

fn main() -> terminal_testlib::Result<()> {
    // Example code here
    Ok(())
}
```

## Troubleshooting

### Example won't compile

- Check feature flags: `cargo run --example name --all-features`
- Verify dependencies: `cargo update`
- Check Rust version: `rustc --version` (MSRV: 1.75)

### Example hangs or times out

- Some examples expect specific applications to be installed
- Check example documentation for prerequisites
- Increase timeout in example code if needed

### Snapshot test fails

- Review changes: `cargo insta review`
- Check if behavior changed intentionally
- Regenerate if needed: `cargo insta test --review --accept`

## Related Documentation

- [Core Documentation](../README.md)
- [API Reference](https://docs.rs/terminal-testlib)
- [Cookbooks](../docs/versions/vNEXT/cookbooks/) - Project-specific patterns
- [Contributing Guide](../CONTRIBUTING.md)

## Questions?

For questions about examples:
- Check example source code comments
- Review related cookbook if available
- Open an issue on GitHub

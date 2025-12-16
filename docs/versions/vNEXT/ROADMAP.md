# terminal-testlib Roadmap - vNEXT

This document outlines planned features and enhancements for future versions of terminal-testlib.

## Table of Contents

- [Library Features](#library-features)
- [CI/Release Enhancements](#cirelease-enhancements)
- [Documentation](#documentation)
- [Timeline](#timeline)

## Library Features

### Async Redraw Support

**Status**: Planned

**Description**: Enhanced async integration for better control over redraw timing and synchronization.

**Proposed API**:
```rust
use terminal_testlib::AsyncTuiTestHarness;

#[tokio::test]
async fn test_async_redraw() -> Result<()> {
    let mut harness = AsyncTuiTestHarness::new(80, 24).await?;

    // Trigger redraw explicitly
    harness.request_redraw().await?;

    // Wait for next redraw with timeout
    harness.wait_for_redraw(Duration::from_millis(100)).await?;

    // Get redraw count
    let count = harness.redraw_count();

    Ok(())
}
```

**Benefits**:
- Better control over render timing in async contexts
- Ability to verify redraw frequency
- Debugging aid for performance testing

**Tracking**: TBD

---

### Time-Travel Snapshotting

**Status**: Planned

**Description**: Capture and restore terminal state at specific points in time for debugging and testing.

**Proposed API**:
```rust
use terminal_testlib::{TuiTestHarness, Snapshot};

#[test]
fn test_time_travel() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;

    // Capture snapshot at point 1
    let snapshot1 = harness.capture_snapshot()?;

    // Make changes
    harness.send_key(KeyCode::Down)?;

    // Capture snapshot at point 2
    let snapshot2 = harness.capture_snapshot()?;

    // Restore to point 1
    harness.restore_snapshot(&snapshot1)?;

    // Verify state is back to point 1
    assert_eq!(harness.screen_text(), snapshot1.screen_text());

    Ok(())
}
```

**Benefits**:
- Reproduce specific states for debugging
- Test undo/redo functionality
- Verify state transitions

**Implementation Notes**:
- Store full terminal buffer state
- Include cursor position and attributes
- Support serialization for persistence

**Tracking**: TBD

---

### Layout Diff Visualization

**Status**: Planned

**Description**: Visual comparison of layout changes between test runs for regression detection.

**Proposed API**:
```rust
use terminal_testlib::{TuiTestHarness, LayoutDiff};

#[test]
fn test_layout_diff() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;

    // Capture initial layout
    let before = harness.capture_layout()?;

    // Make changes
    harness.send_key(KeyCode::Tab)?;

    // Capture new layout
    let after = harness.capture_layout()?;

    // Generate diff
    let diff = LayoutDiff::compare(&before, &after)?;

    // Assert specific regions changed
    assert!(diff.region_changed(Rect::new(0, 5, 10, 5)));
    assert!(!diff.region_changed(Rect::new(0, 0, 80, 3))); // Header unchanged

    // Generate visual diff report
    diff.save_visual_report("target/layout-diff.html")?;

    Ok(())
}
```

**Benefits**:
- Visual regression detection
- Highlight unexpected layout changes
- Debug layout calculations

**Implementation Notes**:
- Use ratatui's `Rect` for region definitions
- Generate HTML reports with side-by-side comparison
- Highlight changed cells with color coding

**Tracking**: TBD

---

### Widget Fixtures

**Status**: Planned

**Description**: Pre-built test fixtures for common ratatui widgets with sample data.

**Proposed API**:
```rust
use terminal_testlib::fixtures::{TableFixture, ListFixture, PopupFixture};
use ratatui::widgets::Table;

#[test]
fn test_table_widget() -> Result<()> {
    // Create table with sample data
    let table_fixture = TableFixture::builder()
        .columns(vec!["ID", "Name", "Status"])
        .rows(10)
        .selected(3)
        .build()?;

    let table: Table = table_fixture.into();

    // Test with harness
    let mut harness = TuiTestHarness::new(80, 24)?;
    // ... test table rendering

    Ok(())
}

#[test]
fn test_popup() -> Result<()> {
    let popup_fixture = PopupFixture::builder()
        .title("Confirm")
        .message("Are you sure?")
        .buttons(vec!["Yes", "No"])
        .size(40, 10)
        .build()?;

    // ... test popup
    Ok(())
}
```

**Available Fixtures**:
- `TableFixture` - Tables with configurable rows/columns
- `ListFixture` - Lists with various item counts
- `PopupFixture` - Modal dialogs and popups
- `FormFixture` - Input forms with validation
- `ChartFixture` - Charts and graphs with sample data

**Tracking**: TBD

---

### Event Scripting DSL

**Status**: Planned

**Description**: Domain-specific language for describing complex user interaction sequences.

**Proposed API**:
```rust
use terminal_testlib::scripting::{EventScript, event_script};

#[test]
fn test_navigation_flow() -> Result<()> {
    let mut harness = TuiTestHarness::new(80, 24)?;

    // Define event script
    let script = event_script! {
        wait_for("Main Menu");
        key(Down);
        key(Down);
        key(Enter);
        wait_for("Settings");
        type_text("username");
        key(Tab);
        type_text("password");
        key(Enter);
        wait_for("Saved");
    };

    // Execute script
    harness.execute_script(&script).await?;

    // Verify final state
    assert!(harness.contains_text("Successfully saved"));

    Ok(())
}

// Or load from file
#[test]
fn test_from_script_file() -> Result<()> {
    let script = EventScript::load("tests/scripts/login-flow.script")?;
    harness.execute_script(&script).await?;
    Ok(())
}
```

**Script File Format**:
```
# login-flow.script
wait_for "Login Screen"
type "admin"
key Tab
type "password123"
key Enter
wait_for "Dashboard" timeout=5s
assert_contains "Welcome, admin"
```

**Tracking**: TBD

---

## CI/Release Enhancements

### Release Artifact Documentation

**Status**: In Progress (see #35)

**Current**: Release workflow creates GitHub release, publishes to crates.io, and builds docs.

**Enhancements**:
- Document all release artifacts in workflow output
- Include example snapshots in release assets
- Generate release notes from changelog
- Archive test results

**Tracking**: #35

---

### Contract Testing - Ratatui Version Matrix

**Status**: Planned

**Description**: Test against multiple versions of ratatui to ensure compatibility.

**Proposed CI Configuration**:
```yaml
name: Ratatui Compatibility

on:
  schedule:
    - cron: '0 0 * * 0'  # Weekly
  workflow_dispatch:

jobs:
  compatibility:
    name: Test with Ratatui ${{ matrix.ratatui-version }}
    runs-on: ubuntu-latest
    strategy:
      matrix:
        ratatui-version:
          - '0.25.0'  # Current stable
          - '0.24.0'  # Previous stable
          - '0.23.0'  # Two versions back
          - 'main'    # Development version
    steps:
      - uses: actions/checkout@v4

      - name: Update ratatui version
        run: |
          cargo add ratatui@${{ matrix.ratatui-version }}

      - name: Run compatibility tests
        run: cargo test --all-features

      - name: Report results
        if: failure()
        run: |
          echo "Compatibility issue with ratatui ${{ matrix.ratatui-version }}"
```

**Benefits**:
- Catch breaking changes early
- Document supported ratatui versions
- Guide users on compatibility

**Tracking**: TBD

---

## Documentation

### Versioned Cookbooks

**Status**: Planned

**Description**: Project-specific guides showing terminal-testlib patterns for different use cases.

**Planned Cookbooks**:

#### 1. Scarab - File Manager Patterns
`docs/versions/vNEXT/cookbooks/scarab.md`

Topics:
- File list navigation testing
- Directory tree navigation
- File operation confirmations
- Dual-pane layout testing

#### 2. Scarab-Nav - Navigation Patterns
`docs/versions/vNEXT/cookbooks/scarab-nav.md`

Topics:
- Vim-style navigation testing
- Modal dialogs
- Quick action menus
- Navigation state verification

#### 3. Tolaria - MTG Deck Manager Patterns
`docs/versions/vNEXT/cookbooks/tolaria.md`

Topics:
- Card list filtering
- Deck validation
- Search functionality
- Data-heavy table testing

#### 4. Sparky - Electric Vehicle Dashboard Patterns
`docs/versions/vNEXT/cookbooks/sparky.md`

Topics:
- Real-time data updates
- Gauge and chart testing
- Alert/notification testing
- Multi-screen workflows

**Structure for Each Cookbook**:
```markdown
# Project Name - Testing Patterns

## Overview
Brief description of the project and testing goals

## Setup
Project-specific test harness configuration

## Common Patterns

### Pattern 1: [Name]
- Use case description
- Code example
- Expected behavior
- Common pitfalls

### Pattern 2: [Name]
...

## Real-World Examples
Complete test examples from the actual project

## Troubleshooting
Common issues and solutions
```

**Tracking**: TBD

---

### Examples Gallery with Golden Snapshots

**Status**: Planned

**Description**: Curated examples demonstrating each feature with golden snapshot tests.

**Proposed Structure**:
```
examples/
├── README.md                          # Gallery index
├── snapshots/                         # Golden snapshots
│   ├── basic_navigation.snap
│   ├── table_widget.snap
│   ├── popup_modal.snap
│   └── ...
├── 01_basic_navigation.rs             # Simple navigation
├── 02_table_widget.rs                 # Table testing
├── 03_popup_modal.rs                  # Modal dialog
├── 04_form_validation.rs              # Form input
├── 05_async_updates.rs                # Async patterns
├── 06_sixel_graphics.rs               # Graphics testing
├── 07_bevy_integration.rs             # Bevy ECS
└── 08_time_travel_debugging.rs        # Advanced debugging
```

**Example Gallery README**:
```markdown
# terminal-testlib Examples Gallery

## Index

| Example | Features | Difficulty | Snapshot |
|---------|----------|------------|----------|
| [Basic Navigation](01_basic_navigation.rs) | PTY, Wait, Keys | Beginner | ✅ |
| [Table Widget](02_table_widget.rs) | Fixtures, Grid | Beginner | ✅ |
| [Popup Modal](03_popup_modal.rs) | Layering, Focus | Intermediate | ✅ |
...

## Running Examples

```bash
# Run a specific example
cargo run --example 01_basic_navigation --all-features

# Run with snapshot update
cargo insta test --example 02_table_widget --review
```

## Viewing Snapshots

Snapshots are stored in `examples/snapshots/` and can be reviewed with:
```bash
cargo insta review
```
```

**Tracking**: TBD

---

## Timeline

### Phase 1: Foundation (Current)
- ✅ Core PTY testing
- ✅ Sixel graphics support
- ✅ Bevy ECS integration
- ✅ Async tokio support
- ✅ Snapshot testing
- ✅ Headless mode

### Phase 2: Documentation & Process (In Progress)
- ✅ Documentation versioning (#34)
- ✅ Release pipeline documentation (#35)
- 🚧 Roadmap scaffolding (#36)

### Phase 3: Advanced Features (Planned)
- ⏳ Async redraw support
- ⏳ Time-travel snapshotting
- ⏳ Layout diff visualization
- ⏳ Widget fixtures
- ⏳ Event scripting DSL

### Phase 4: Testing & Quality (Planned)
- ⏳ Ratatui version matrix testing
- ⏳ Expanded example gallery
- ⏳ Golden snapshot tests

### Phase 5: Documentation (Planned)
- ⏳ Project cookbooks (scarab, tolaria, etc.)
- ⏳ Migration guides
- ⏳ Video tutorials

## Contributing to Roadmap

To suggest new features or changes to the roadmap:

1. Open an issue with the `enhancement` label
2. Describe the feature and use case
3. Provide code examples if possible
4. Discuss with maintainers

For roadmap updates:
- Update this document in a PR
- Reference related issues
- Update status as features progress

## Related Documentation

- [docs/STRUCTURE.md](../../STRUCTURE.md) - Documentation organization
- [docs/RELEASE.md](../../RELEASE.md) - Release process
- [CHANGELOG.md](../../../CHANGELOG.md) - Version history

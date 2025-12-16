# Terminal Profiles

Terminal profiles enable testing your TUI application across different terminal emulators to ensure compatibility. Each profile represents a specific terminal with its capabilities (colors, graphics protocols, mouse support, etc.).

## Overview

Different terminal emulators support different features:
- **Color depth**: Monochrome, 8, 16, 256, or true color (16.7M colors)
- **Unicode support**: ASCII only vs full UTF-8 with wide characters
- **Mouse protocols**: X10, VT200, SGR, UTF-8
- **Graphics protocols**: Sixel, iTerm2 inline images, Kitty graphics
- **Modern features**: Synchronized output, bracketed paste, focus events

## Quick Start

```rust
use terminal_testlib::{TuiTestHarness, TerminalProfile, Feature};

// Create a harness configured for WezTerm
let harness = TuiTestHarness::new(80, 24)?
    .with_terminal_profile(TerminalProfile::WezTerm);

// Check if Sixel is supported
if harness.supports_feature(Feature::Sixel) {
    // Run Sixel-specific tests
}

// Get full capabilities
let caps = harness.terminal_capabilities();
println!("Color depth: {:?}", caps.color_depth);
```

## Available Terminal Profiles

### Legacy Terminals

#### VT100
The classic DEC VT100 terminal.
- **Color depth**: Monochrome
- **Unicode**: No
- **Mouse**: None
- **Graphics**: None
- **TERM**: `vt100`

**Use case**: Testing minimal compatibility, ensuring your app works in constrained environments.

### Basic xterm Variants

#### Xterm256
Standard xterm with 256 colors.
- **Color depth**: 256 colors
- **Unicode**: Yes (UTF-8)
- **Mouse**: VT200
- **Graphics**: None
- **TERM**: `xterm-256color`

**Use case**: Testing with basic modern terminal features.

#### XtermTrueColor
Modern xterm with true color support.
- **Color depth**: True color (24-bit)
- **Unicode**: Yes (UTF-8 + wide chars)
- **Mouse**: SGR (extended coordinates)
- **Graphics**: None
- **TERM**: `xterm-256color`

**Use case**: Testing modern color support without graphics protocols.

### Terminal Multiplexers

#### Screen
GNU Screen terminal multiplexer.
- **Color depth**: 256 colors
- **Unicode**: Yes (UTF-8)
- **Mouse**: X10 (basic)
- **Graphics**: None
- **TERM**: `screen`

**Use case**: Testing inside GNU Screen sessions.

#### Tmux
Modern terminal multiplexer.
- **Color depth**: 256 colors
- **Unicode**: Yes (UTF-8 + wide chars)
- **Mouse**: SGR
- **Graphics**: None
- **TERM**: `tmux-256color`

**Use case**: Testing inside tmux sessions, common in development workflows.

### Linux Desktop Terminals

#### Konsole
KDE's terminal emulator.
- **Color depth**: True color
- **Unicode**: Yes (UTF-8 + wide chars)
- **Mouse**: SGR
- **Graphics**: None
- **TERM**: `konsole-256color`

**Use case**: Testing on KDE desktop environments.

#### GnomeTerminal
GNOME's default terminal.
- **Color depth**: True color
- **Unicode**: Yes (UTF-8 + wide chars)
- **Mouse**: SGR
- **Graphics**: None
- **TERM**: `xterm-256color`

**Use case**: Testing on GNOME desktop environments.

### Modern GPU-Accelerated Terminals

#### Alacritty
Fast, GPU-accelerated terminal emulator.
- **Color depth**: True color
- **Unicode**: Yes (UTF-8 + wide chars)
- **Mouse**: SGR
- **Graphics**: None
- **Synchronized output**: Yes
- **TERM**: `alacritty`

**Use case**: Testing modern terminal without graphics protocols, common among developers.

#### Kitty
GPU-accelerated with custom graphics protocol.
- **Color depth**: True color
- **Unicode**: Yes (UTF-8 + wide chars)
- **Mouse**: SGR
- **Graphics**: Kitty graphics protocol
- **TERM**: `xterm-kitty`

**Use case**: Testing Kitty-specific graphics rendering.

### Terminals with Sixel Support

#### WezTerm
Modern terminal with extensive protocol support.
- **Color depth**: True color
- **Unicode**: Yes (UTF-8 + wide chars)
- **Mouse**: SGR
- **Graphics**: Sixel + iTerm2 inline images
- **All modern features**: Yes
- **TERM**: `wezterm`

**Use case**: Testing full modern terminal capabilities including Sixel.

#### ITerm2
macOS terminal with inline image support.
- **Color depth**: True color
- **Unicode**: Yes (UTF-8 + wide chars)
- **Mouse**: SGR
- **Graphics**: Sixel + iTerm2 inline images
- **TERM**: `xterm-256color`

**Use case**: Testing on macOS with graphics support.

### Other Platforms

#### WindowsTerminal
Microsoft's modern Windows terminal.
- **Color depth**: True color
- **Unicode**: Yes (UTF-8 + wide chars)
- **Mouse**: SGR
- **Graphics**: None
- **TERM**: `xterm-256color`

**Use case**: Testing on Windows 10/11.

#### VSCode
Visual Studio Code integrated terminal.
- **Color depth**: True color
- **Unicode**: Yes (UTF-8 + wide chars)
- **Mouse**: SGR
- **Graphics**: None
- **Limited features**: No focus events, title setting
- **TERM**: `xterm-256color`

**Use case**: Testing in VS Code's integrated terminal.

### Testing Profiles

#### Minimal
Bare minimum features for compatibility testing.
- **Color depth**: 16 colors
- **Unicode**: No
- **Mouse**: None
- **Graphics**: None
- **TERM**: `xterm`

**Use case**: Ensure your app works in extremely constrained environments.

#### Maximum
All features enabled for comprehensive testing.
- **Color depth**: True color
- **Unicode**: Yes (UTF-8 + wide chars)
- **Mouse**: SGR
- **Graphics**: All protocols (Sixel, Kitty, iTerm2)
- **All modern features**: Yes
- **TERM**: `xterm-256color`

**Use case**: Testing all features together, regression testing.

## Usage Patterns

### Basic Profile Selection

```rust
use terminal_testlib::{TuiTestHarness, TerminalProfile};

// Direct profile selection
let harness = TuiTestHarness::new(80, 24)?
    .with_terminal_profile(TerminalProfile::WezTerm);
```

### TERM Value Simulation

```rust
// Simulate by TERM environment variable
let harness = TuiTestHarness::new(80, 24)?
    .simulate_terminfo("xterm-256color");

// Case insensitive
let harness = TuiTestHarness::new(80, 24)?
    .simulate_terminfo("WEZTERM");
```

### Feature Checking

```rust
use terminal_testlib::Feature;

if harness.supports_feature(Feature::Sixel) {
    // Run Sixel graphics tests
}

if harness.supports_feature(Feature::TrueColor) {
    // Test 24-bit color rendering
}

if harness.supports_feature(Feature::MouseSGR) {
    // Test extended mouse coordinates
}
```

### Getting Full Capabilities

```rust
let caps = harness.terminal_capabilities();

println!("Terminal: {}", caps.term_name);
println!("Color depth: {:?}", caps.color_depth);
println!("Sixel: {}", caps.sixel_support);
println!("Mouse: {:?}", caps.mouse_protocol);

// Pretty-print all capabilities
println!("{}", caps.summary());
```

### Testing Across Multiple Profiles

```rust
use terminal_testlib::TerminalProfile;

#[test]
fn test_app_works_on_all_terminals() -> Result<()> {
    let profiles = vec![
        TerminalProfile::VT100,
        TerminalProfile::Xterm256,
        TerminalProfile::Alacritty,
        TerminalProfile::WezTerm,
    ];

    for profile in profiles {
        let mut harness = TuiTestHarness::new(80, 24)?
            .with_terminal_profile(profile);

        // Spawn your app
        harness.spawn(CommandBuilder::new("./my-app"))?;

        // Test basic functionality
        harness.wait_for_text("Welcome")?;

        // Conditional feature tests
        if harness.supports_feature(Feature::TrueColor) {
            // Verify true color rendering
        }
    }

    Ok(())
}
```

### Builder Pattern

```rust
let harness = TuiTestHarness::builder()
    .with_size(100, 30)
    .with_terminal_profile(TerminalProfile::WezTerm)
    .with_timeout(Duration::from_secs(10))
    .build()?;
```

## Feature Types

### Color Features
- `Feature::Colors256` - 256-color palette
- `Feature::TrueColor` - 24-bit RGB colors

### Text Features
- `Feature::Unicode` - UTF-8 support
- `Feature::WideCharacters` - Emoji, CJK character support

### Mouse Features
- `Feature::MouseX10` - Basic mouse (press/release)
- `Feature::MouseVT200` - Mouse with modifiers
- `Feature::MouseSGR` - Extended coordinates (1006)
- `Feature::MouseUTF8` - UTF-8 mouse encoding (1005)
- `Feature::MouseMotion` - Motion tracking

### Graphics Features
- `Feature::Sixel` - Sixel graphics protocol
- `Feature::KittyGraphics` - Kitty graphics protocol
- `Feature::ITerm2Images` - iTerm2 inline images

### Modern Features
- `Feature::BracketedPaste` - Safe paste mode
- `Feature::SynchronizedOutput` - Flicker-free updates (2026)
- `Feature::AlternateScreen` - Alternate screen buffer
- `Feature::SetTitle` - Window title setting
- `Feature::FocusEvents` - Focus in/out notifications

## Color Depth Hierarchy

Color depth levels are ordered:

```rust
ColorDepth::Monochrome < ColorDepth::Colors8 < ColorDepth::Colors16
    < ColorDepth::Colors256 < ColorDepth::TrueColor
```

When checking color features:
- `supports(Feature::Colors256)` returns true for 256-color and true color terminals
- `supports(Feature::TrueColor)` returns true only for true color terminals

Example:

```rust
let caps = TerminalProfile::WezTerm.capabilities();
assert!(caps.supports(Feature::Colors256));   // true
assert!(caps.supports(Feature::TrueColor));   // true

let caps = TerminalProfile::Xterm256.capabilities();
assert!(caps.supports(Feature::Colors256));   // true
assert!(caps.supports(Feature::TrueColor));   // false
```

## Mouse Protocol Hierarchy

Mouse protocols build on each other:

1. **None** - No mouse support
2. **X10** - Basic button press/release
3. **VT200** - X10 + modifier keys
4. **SGR/UTF8** - VT200 + extended coordinates

When a terminal supports a higher-level protocol, it typically supports all lower levels.

## Best Practices

### 1. Test Minimal Compatibility First

```rust
#[test]
fn test_minimal_terminal_support() -> Result<()> {
    let harness = TuiTestHarness::new(80, 24)?
        .with_terminal_profile(TerminalProfile::Minimal);

    // Your app should work even with minimal features
    // Test core functionality without colors, unicode, etc.
}
```

### 2. Graceful Feature Degradation

```rust
fn render_logo(harness: &TuiTestHarness) -> Result<()> {
    if harness.supports_feature(Feature::Sixel) {
        // Render Sixel logo
    } else if harness.supports_feature(Feature::Unicode) {
        // Render Unicode art logo
    } else {
        // Render ASCII art logo
    }
    Ok(())
}
```

### 3. Document Required Features

```rust
/// This test requires Sixel support
#[test]
fn test_image_gallery() -> Result<()> {
    let harness = TuiTestHarness::new(80, 24)?
        .with_terminal_profile(TerminalProfile::WezTerm);

    assert!(harness.supports_feature(Feature::Sixel),
            "This test requires Sixel support");

    // ... test code ...
}
```

### 4. CI/CD Testing

In CI environments, test with a representative set:

```rust
#[test]
fn test_common_terminals() -> Result<()> {
    // Test terminals commonly used in development
    let profiles = vec![
        TerminalProfile::Xterm256,      // Basic modern terminal
        TerminalProfile::Alacritty,     // Popular developer terminal
        TerminalProfile::Tmux,          // Common in workflows
        TerminalProfile::VSCode,        // IDE users
    ];

    for profile in profiles {
        let harness = TuiTestHarness::new(80, 24)?
            .with_terminal_profile(profile);

        test_basic_functionality(&harness)?;
    }

    Ok(())
}
```

### 5. Profile-Specific Tests

```rust
#[test]
fn test_sixel_graphics() -> Result<()> {
    // Only run on terminals with Sixel support
    let profiles = vec![
        TerminalProfile::WezTerm,
        TerminalProfile::ITerm2,
    ];

    for profile in profiles {
        let harness = TuiTestHarness::new(80, 24)?
            .with_terminal_profile(profile);

        assert!(harness.supports_feature(Feature::Sixel));

        // Test Sixel-specific functionality
    }

    Ok(())
}
```

## Custom Capabilities

You can extend capabilities with custom fields:

```rust
let mut caps = TerminalProfile::WezTerm.capabilities();
caps.custom.insert("vendor".to_string(), "wez".to_string());
caps.custom.insert("version".to_string(), "20230712".to_string());

if let Some(vendor) = caps.custom.get("vendor") {
    println!("Terminal vendor: {}", vendor);
}
```

## Profile Lookup

Get a profile by name:

```rust
let profile = TerminalProfile::from_name("wezterm");
assert_eq!(profile, Some(TerminalProfile::WezTerm));

// Also works with TERM values
let profile = TerminalProfile::from_name("xterm-256color");
assert_eq!(profile, Some(TerminalProfile::Xterm256));

// Case insensitive
let profile = TerminalProfile::from_name("ALACRITTY");
assert_eq!(profile, Some(TerminalProfile::Alacritty));

// Unknown terminals return None
let profile = TerminalProfile::from_name("unknown");
assert_eq!(profile, None);
```

## List All Profiles

```rust
let all = TerminalProfile::all();
for profile in all {
    println!("{:<20} (TERM={})",
             profile.display_name(),
             profile.term_name());
}
```

## Examples

See `examples/terminal_profiles_demo.rs` for a comprehensive demonstration of all features.

Run with:
```bash
cargo run --example terminal_profiles_demo --features sixel
```

## Related Documentation

- [Testing Guide](./TESTING.md) - General testing approaches
- [Sixel Testing](./SIXEL.md) - Graphics protocol testing
- [API Documentation](https://docs.rs/terminal-testlib) - Full API reference

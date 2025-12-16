# Research Findings: Terminal Testing Solutions

## Overview

This document summarizes research into existing terminal emulator testing solutions, focusing on libraries and approaches that can facilitate integration testing of Ratatui TUI applications, particularly for features like Sixel rendering that require actual terminal escape sequence processing.

## Existing Rust Terminal Parsing Libraries

### 1. VTE (from Alacritty)

**Repository**: https://github.com/alacritty/vte
**Crates.io**: vte (875,909 downloads/month, used in 1,015 crates)
**Latest Version**: 0.15.0 (as of 2025)

#### Key Features
- Parser for implementing virtual terminal emulators in Rust
- Implements Paul Williams' ANSI parser state machine
- Table-driven parser with minimal branching (excellent performance)
- Does NOT assign semantic meaning to parsed data
- Requires implementing the `Perform` trait to handle actions

#### Architecture
```rust
// Pseudo-code example
struct Parser {
    // State machine book keeping
}

trait Perform {
    // Handle escape sequences, control codes, etc.
}
```

The parser handles the state transitions, while the `Perform` trait implementor decides what to do with the parsed data.

#### Testing Tools
- **vtebench**: Maintained by Alacritty for quantifying terminal emulator throughput
- Alacritty consistently scores better than competition using vtebench

#### Strengths
- Battle-tested in production (Alacritty)
- Extremely performant
- Well-maintained and popular

#### Weaknesses
- Low-level: requires significant implementation work
- No built-in screen buffer or terminal state management

### 2. vt100-rust

**Repository**: https://github.com/doy/vt100-rust
**Crates.io**: vt100

#### Key Features
- Parses terminal byte stream
- Provides in-memory representation of rendered contents
- Essentially the terminal parser component of a graphical terminal emulator extracted as a library
- Higher-level than VTE - includes screen buffer management

#### Architecture
- Takes a stream of bytes
- Maintains internal state of what the terminal would display
- Allows querying the current screen state

#### Use Case
Perfect for testing: you can feed it your application's output and then query what would be displayed on screen.

#### Strengths
- Higher-level abstraction than VTE
- Built-in screen state management
- Designed specifically for the "parse and verify" use case

#### Weaknesses
- Less popular than VTE
- May not support all modern terminal features

### 3. termwiz (from WezTerm)

**Repository**: https://github.com/wez/wezterm/tree/HEAD/termwiz
**Part of**: WezTerm workspace

#### Key Features
- Number of support functions for terminal applications
- Escape sequence parsing
- Terminal attributes support
- Used by WezTerm internally

#### Relationship to vtparse
- **vtparse**: Lowest level parser, categorizes sequence types without semantic meaning
- **termwiz**: Higher level, built on top of parsing infrastructure

#### Strengths
- Part of WezTerm ecosystem
- Actively maintained
- Comprehensive terminal feature support

#### Weaknesses
- Tightly coupled to WezTerm
- May include more than needed for simple testing

## PTY (Pseudo-Terminal) Libraries

### portable-pty

**Repository**: Part of WezTerm
**Crates.io**: portable-pty

#### Key Features
- Cross-platform API for pseudo-terminal interfaces
- Runtime-selectable implementations via traits
- Works on Linux, macOS, and Windows

#### Basic Usage
```rust
// Create PTY with specified size
let pty_pair = pty_system.openpty(PtySize { ... })?;

// Spawn command
let mut child = pty_pair.slave.spawn_command(CommandBuilder::new("bash"))?;

// Read output
let mut reader = pty_pair.master.try_clone_reader()?;

// Write data
writeln!(pty_pair.master, "ls")?;
```

#### Common Pitfalls
- Programs may hang when trying to read PTY output
- Need to handle buffering correctly
- Programs detect TTY vs non-TTY and behave differently

#### Use Cases for Testing
- Create virtual terminal that tricks programs into emitting terminal sequences
- Capture and parse sequences in real-time
- Essential for integration testing CLI tools that need TTY

#### Strengths
- Cross-platform
- Part of well-maintained WezTerm project
- Actively used in production

### Other PTY Libraries
- **pty-rs**: Pseudoterminal Rust library (less popular)
- **pty**: Basic PTY support (older, less maintained)

## Snapshot Testing Frameworks

### expect-test

**Repository**: https://github.com/rust-analyzer/expect-test
**Crates.io**: expect-test

#### Key Features
- Minimalistic snapshot testing for Rust
- Stores snapshots inside source code
- Can automatically update snapshots
- Run with `UPDATE_EXPECT=1` to update snapshots

#### Ideal for TUI Testing
- Better fit than standard assertions for TUIs
- Functional requirements aren't easy to express with assert statements
- Can capture and compare terminal output

### insta

**Crates.io**: insta
**Popularity**: Most popular snapshot testing framework in Rust

#### Key Features
- Batteries-included snapshot testing
- Multiple snapshot serialization formats
- Command-line tool for reviewing and recording outputs
- More feature-rich than expect-test

#### Use Case
- Complex snapshot comparisons
- Need for review workflow
- Multiple output formats

## Testing Approaches

### 1. "Expect"-Style Testing

**Origin**: The expect(1) program that "talks" to interactive programs according to a script

**Concept**:
- Know what can be expected from a program
- Know what the correct response should be
- Script the interaction
- Verify responses

**Modern Application**:
- Use PTY to communicate with application
- Send input sequences
- Parse and verify output

### 2. Screen Buffer Verification

**Approach**:
- Use terminal emulator library (vt100, VTE+custom screen buffer)
- Feed application output to emulator
- Query the resulting screen state
- Compare against expected state (snapshot testing)

**Challenge**: "When constructing a PTY, you only get a bidirectional stream of bytes, not an actual picture of the screen, so in automated testing you have to interpret escape codes and render screen contents yourself"

**Solution**: Combine PTY + terminal emulator library

### 3. Headless Testing

**Examples**:
- Ratatui.cs (.NET bindings) provides headless snapshot rendering for CI
- No terminal required
- Deterministic results

**Implementation for Rust**:
- Use alternative backend for Ratatui
- Capture rendering without actual terminal
- r3bl_tui demonstrates swapping input/output devices

## Sixel Testing Specifics

### Detection Tools

#### lsix
**Repository**: https://github.com/hackerb9/lsix
**Function**: Shows thumbnails in terminal using sixel graphics

**Testing Method**:
```bash
lsix
# If not supported: "Error: Your terminal does not report having sixel graphics support."
```

#### Are We Sixel Yet
**Website**: https://www.arewesixelyet.com/
**Purpose**: Tracks which terminals support sixel

### Validation Suites

#### Jexer Sixel Tests
**Website**: https://jexer.sourceforge.io/sixel.html

**Purpose**:
- Session captures of Jexer-style sixel output
- Aid terminal authors testing their sixel implementation
- Includes raw output + screenshot of expected result under xterm

### Test Images
- "snake" image from libsixel project
- "map8.six" sample image from libsixel repository

### Programmatic Detection

**Julia (Sixel.jl)**:
```julia
Sixel.is_sixel_supported()
```

**R (terminalgraphics)**:
- Auto-detect sixel support
- Issue warning if not supported but still output

### Reference Implementation

#### libsixel
**Website**: https://saitoha.github.io/libsixel/
**Purpose**: The new standard of SIXEL development

## WezTerm-Specific Resources

### Architecture

WezTerm is organized as a Cargo workspace with multiple crates:

- **wezterm-gui**: Graphical UI binary (terminal emulation + rendering + multiplexer)
- **wezterm-mux-server**: Standalone multiplexer daemon (can run headless)
- **termwiz**: Terminal support functions library
- **vtparse**: Low-level parser
- **portable-pty**: Cross-platform PTY support

### Headless Multiplexer

**wezterm-mux-server** can run headless and accept client connections:
- Unix domain sockets
- TLS connections

**Potential for Testing**:
- Could potentially run wezterm-mux-server in test environment
- Connect test client
- Verify terminal output

## Key Insights for Library Design

1. **Two-Layer Approach Needed**:
   - PTY layer: portable-pty for creating virtual terminals
   - Parsing layer: vt100 or VTE+custom screen buffer for interpreting output

2. **Snapshot Testing is Essential**:
   - Terminal output is visual and complex
   - Snapshot testing (expect-test or insta) is better than assertions
   - Need to capture "what the screen looks like" not just raw bytes

3. **Sixel Requires Special Handling**:
   - Need terminal emulator that implements sixel protocol
   - Test images and validation suites available
   - Can verify both protocol support and rendering correctness

4. **WezTerm Components are Modular**:
   - termwiz and portable-pty are already extracted as reusable crates
   - Could leverage these directly
   - WezTerm mux-server could potentially be used for integration tests

5. **Headless vs PTY Trade-offs**:
   - Headless: Faster, more deterministic, but requires alternative backend
   - PTY: Tests the actual terminal integration, but more complex setup

## Recommendations for terminal-testlib Library

Based on this research, the library should:

1. **Use portable-pty** for PTY creation (proven, cross-platform, maintained)

2. **Use vt100 crate** for terminal emulation (higher-level, easier to work with than raw VTE)

3. **Integrate snapshot testing** via expect-test or insta

4. **Provide helper traits** for common testing patterns:
   - Spawn TUI app in PTY
   - Capture screen state
   - Send input events
   - Assert on visual output

5. **Special Sixel support**:
   - Helpers for loading test images
   - Verification against known-good output
   - Integration with Jexer test suite

6. **Async support** for modern Rust applications

7. **Examples and test utilities** for common Ratatui patterns

## References

- [Alacritty VTE Parser](https://github.com/alacritty/vte)
- [vt100-rust Terminal Emulator](https://github.com/doy/vt100-rust)
- [WezTerm Terminal Emulator](https://github.com/wezterm/wezterm)
- [portable-pty Documentation](https://docs.rs/portable-pty/latest/portable_pty/index.html)
- [Testing TUI Apps Blog Post](https://blog.waleedkhan.name/testing-tui-apps/)
- [Integration Testing TUI in Rust](https://quantonganh.com/2024/01/21/integration-testing-tui-app-in-rust.md)
- [Are We Sixel Yet](https://www.arewesixelyet.com/)
- [Jexer Sixel Tests](https://jexer.sourceforge.io/sixel.html)
- [libsixel](https://saitoha.github.io/libsixel/)

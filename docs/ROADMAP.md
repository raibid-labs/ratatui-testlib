# term-test Implementation Roadmap

## Project Vision

Create a Rust library for integration testing of terminal user interface applications, with first-class support for Ratatui, Bevy ECS integration, and graphics protocols like Sixel. **Primary goal: Enable comprehensive testing for the dgx-pixels project.**

## MVP Definition (Based on dgx-pixels Requirements)

The **Minimum Viable Product** must support the dgx-pixels project's testing needs:

1. ✅ Headless terminal emulation (CI/CD compatible)
2. ✅ Sixel graphics position verification and bounds checking
3. ✅ Sixel clearing validation on screen transitions
4. ✅ Bevy ECS integration (query entities, control update cycles)
5. ✅ bevy_ratatui plugin support
6. ✅ Text input and cursor position testing
7. ✅ Tokio async runtime support
8. ✅ Runs in GitHub Actions without X11/Wayland

**Success Criteria**: Can detect and prevent the Sixel positioning and persistence bugs that occurred in dgx-pixels development.

## Phases

### Phase 0: Foundation ✅

**Goal**: Establish project structure, research, and documentation

**Status**: ✅ Complete

**Deliverables**:
- [x] Repository initialization
- [x] Comprehensive research documentation
- [x] Architecture design
- [x] Gap analysis of existing solutions
- [x] Testing approaches documentation
- [x] dgx-pixels requirements analysis
- [x] This roadmap

### Phase 1: Core PTY Harness

**Goal**: Basic PTY-based test harness with screen capture and cursor tracking

**Priority**: P0 (Critical - MVP Blocker)

**Dependencies**: None

**Tasks**:

1. **Project Setup**
   - [ ] Initialize Cargo workspace
   - [ ] Set up CI/CD (GitHub Actions) with headless Linux runner
   - [ ] Configure linting (clippy, rustfmt)
   - [ ] Set up pre-commit hooks
   - [ ] Create CONTRIBUTING.md

2. **PTY Management Layer**
   - [ ] Integrate `portable-pty` crate
   - [ ] Implement `TestTerminal` wrapper
   - [ ] Handle PTY creation and lifecycle
   - [ ] Implement process spawning (for external binaries)
   - [ ] Add read/write operations with buffering
   - [ ] **Test on Linux (primary CI platform)**

3. **Terminal Emulation Layer**
   - [ ] Integrate `vt100` crate
   - [ ] **Validate vt100 cursor position tracking** (critical for Sixel)
   - [ ] Implement `ScreenState` wrapper
   - [ ] Feed PTY output to parser
   - [ ] Expose screen query methods (`contents()`, `cell_at()`)
   - [ ] **Track cursor position** (for Sixel position verification)
   - [ ] Support color and attribute queries

4. **Basic Test Harness**
   - [ ] Implement `TuiTestHarness` struct
   - [ ] Add `new(width, height)` constructor
   - [ ] Add `spawn(Command)` method for external processes
   - [ ] Add `send_text(str)` method
   - [ ] Add simple wait methods (time-based polling)
   - [ ] Add `screen_contents()` method
   - [ ] **Add `get_cursor_position()` method** (MVP requirement)
   - [ ] Implement error types (`TermTestError`)

5. **Testing & Documentation**
   - [ ] Write unit tests for PTY layer
   - [ ] Write integration tests for harness
   - [ ] Create basic usage examples
   - [ ] Write API documentation
   - [ ] Test on Linux (primary platform)

**Success Criteria**:
- Can spawn a simple TUI app in PTY
- Can send text input
- Can capture screen contents and cursor position
- Works on Linux (macOS/Windows nice-to-have)
- Basic examples run successfully
- **CI/CD runs tests headlessly**

**Estimated Effort**: 2-3 weeks

### Phase 2: Event Simulation & Async Support

**Goal**: Rich event simulation and Tokio async integration

**Priority**: P0 (Critical - MVP Blocker)

**Dependencies**: Phase 1

**Tasks**:

1. **Event Simulation**
   - [ ] Implement keyboard event sending (single keys)
   - [ ] Support special keys (arrows, Tab, Enter, Esc, numbers)
   - [ ] Support key sequences (multiple keys)
   - [ ] Add `press_key(KeyCode)` method
   - [ ] Add `type_text(str)` helper (sends character events)
   - [ ] **Test navigation keys for dgx-pixels** (Tab, 1-8, Esc)

2. **Smart Waiting**
   - [ ] Implement condition-based waiting
   - [ ] Add timeout support (configurable, default 5s)
   - [ ] Implement polling mechanism (check condition periodically)
   - [ ] Add `wait_for(condition)` method
   - [ ] Create common condition helpers (contains text, etc.)
   - [ ] Add debugging output for failed waits

3. **Tokio Async Support** (MVP requirement)
   - [ ] Add tokio feature flag
   - [ ] Implement `AsyncTuiTestHarness`
   - [ ] Make spawn, send, wait operations async
   - [ ] Support Tokio runtime in tests
   - [ ] **Test with Tokio-based TUI apps**

4. **Testing & Documentation**
   - [ ] Test keyboard events
   - [ ] Test waiting conditions
   - [ ] Test timeout handling
   - [ ] Write examples for async usage
   - [ ] Document waiting patterns

**Success Criteria**:
- Can simulate keyboard input (keys and text)
- Can wait for screen state conditions with timeout
- Async harness works with Tokio
- **Can test dgx-pixels navigation** (Tab, number keys)
- Examples demonstrate event simulation

**Estimated Effort**: 1-2 weeks

### Phase 3: Sixel Graphics Support with Position Tracking

**Goal**: Enable Sixel testing with position verification and bounds checking

**Priority**: P0 (Critical - MVP Blocker, Original Motivation)

**Dependencies**: Phase 2

**Tasks**:

1. **Sixel Sequence Detection**
   - [ ] **Research vt100 Sixel support** (validate or find alternative)
   - [ ] Detect Sixel escape sequences in output
   - [ ] Parse Sixel escape sequences (structure validation)
   - [ ] Extract Sixel metadata (dimensions, colors)
   - [ ] **Capture cursor position when Sixel is rendered** (critical!)

2. **Sixel Position Tracking** (MVP requirement)
   - [ ] Implement `SixelSequence` type with position
   - [ ] Track bounds (position + dimensions)
   - [ ] Associate Sixel sequences with terminal coordinates
   - [ ] Implement `SixelCapture` type for all sequences
   - [ ] **Support area-bounded queries** (in/outside area)

3. **Sixel Validation** (MVP requirement)
   - [ ] Validate Sixel sequence structure
   - [ ] Implement `assert_sixel_within(area)`
   - [ ] Implement `assert_no_sixel_outside(area)`
   - [ ] Implement `has_sixel_graphics()` check
   - [ ] Implement `capture_sixel_state()` for snapshots
   - [ ] Support clearing detection (compare before/after)

4. **Test Fixtures**
   - [ ] Include dgx-pixels test images
   - [ ] Create reference Sixel output files
   - [ ] Document fixture usage

5. **Testing & Documentation**
   - [ ] Test Sixel detection and parsing
   - [ ] Test position tracking accuracy
   - [ ] Test bounds checking assertions
   - [ ] **Test dgx-pixels preview area scenario**
   - [ ] Create Sixel testing guide

**Success Criteria**:
- Can capture Sixel sequences with positions
- Can verify Sixel within bounds (preview area)
- Can detect Sixel outside bounds
- Can detect Sixel clearing on screen change
- **Can prevent dgx-pixels Sixel bugs**

**Estimated Effort**: 2-3 weeks

**Risk**: vt100 may not support Sixel or position tracking. **Mitigation**: Have termwiz as backup, or extend vt100.

### Phase 4: Bevy ECS Integration

**Goal**: Support testing of Bevy-based TUI applications (bevy_ratatui)

**Priority**: P0 (Critical - MVP Blocker for dgx-pixels)

**Dependencies**: Phase 3

**Tasks**:

1. **Bevy Harness Wrapper**
   - [ ] Add bevy feature flag
   - [ ] Create `BevyTuiTestHarness` struct
   - [ ] Wrap `TuiTestHarness` + Bevy `App`
   - [ ] Support headless Bevy app initialization
   - [ ] Integrate with bevy_ratatui plugin

2. **Update Cycle Control** (MVP requirement)
   - [ ] Implement `update()` to run one Bevy frame
   - [ ] Implement `update_n(count)` to run N frames
   - [ ] Coordinate Bevy updates with terminal rendering
   - [ ] Handle event propagation to Bevy systems

3. **ECS Querying** (MVP requirement)
   - [ ] Implement `query_entities<Component>()`
   - [ ] Implement `get_resource<Resource>()`
   - [ ] Support component inspection
   - [ ] Access Bevy World for custom queries

4. **Event Integration**
   - [ ] Convert keyboard events to Bevy events
   - [ ] Implement `send_bevy_event<Event>()`
   - [ ] Support crossterm event types
   - [ ] Verify event processing in systems

5. **Testing & Documentation**
   - [ ] Test Bevy app lifecycle
   - [ ] Test ECS queries
   - [ ] Test system execution
   - [ ] **Test dgx-pixels Job entity creation**
   - [ ] Create Bevy integration guide

**Success Criteria**:
- Can create headless Bevy TUI test harness
- Can control Bevy update cycles frame-by-frame
- Can query entities and resources
- **Can test dgx-pixels screens and state**
- Examples show Bevy ECS testing patterns

**Estimated Effort**: 2-3 weeks

**Risk**: Bevy headless mode complexity. **Mitigation**: Start with minimal Bevy app, add features incrementally.

### Phase 5: Snapshot Testing & High-Level Assertions

**Goal**: Ergonomic API for common assertions and snapshot testing

**Priority**: P0 (Critical - MVP for Developer Experience)

**Dependencies**: Phase 4

**Tasks**:

1. **Snapshot Support**
   - [ ] Implement `Snapshot` type
   - [ ] Add metadata (size, cursor, timestamp)
   - [ ] Implement text serialization
   - [ ] Implement comparison methods
   - [ ] Generate useful diffs

2. **insta Integration**
   - [ ] Add insta feature flag
   - [ ] Implement insta-compatible serialization
   - [ ] Test with insta snapshots
   - [ ] Create insta examples

3. **High-Level Assertions** (MVP requirement)
   - [ ] Implement `assert_text_at(x, y, text)`
   - [ ] Implement `assert_text_contains(text)`
   - [ ] Implement `assert_area_contains_text(area, text)`
   - [ ] Implement `assert_cursor_position(x, y)`
   - [ ] Implement `assert_cursor_in_area(area)`
   - [ ] Implement `assert_on_screen(screen)` helper

4. **Ratatui Helpers**
   - [ ] Add ratatui feature flag
   - [ ] Create `RatatuiTestHelper` wrapper
   - [ ] Area-based assertions
   - [ ] Layout verification helpers

5. **Testing & Documentation**
   - [ ] Test all assertion methods
   - [ ] Test snapshot workflow
   - [ ] **Create dgx-pixels test examples**
   - [ ] Write assertion cookbook

**Success Criteria**:
- Snapshots work with insta
- Assertions are ergonomic and intuitive
- **dgx-pixels use cases have helpers**
- Examples show best practices

**Estimated Effort**: 1-2 weeks

### Phase 6: Polish & Documentation (MVP Release)

**Goal**: Production-ready MVP for dgx-pixels

**Priority**: P0 (Critical - MVP Completeness)

**Dependencies**: Phase 5

**Tasks**:

1. **Error Handling**
   - [ ] Audit all error types
   - [ ] Improve error messages (actionable)
   - [ ] Add error context (what failed, where)
   - [ ] Create error handling guide

2. **Debug Support** (MVP requirement)
   - [ ] Save terminal state on test failure
   - [ ] Export failed state as ANSI text file
   - [ ] Add verbose logging option (escape sequences)
   - [ ] Document debugging techniques

3. **Documentation**
   - [ ] Complete API documentation (rustdoc)
   - [ ] Write comprehensive user guide
   - [ ] Create cookbook/recipes
   - [ ] **Write dgx-pixels integration guide**
   - [ ] Add troubleshooting section

4. **CI/CD**
   - [ ] Configure GitHub Actions for tests
   - [ ] Test on Ubuntu (primary), macOS/Windows (nice-to-have)
   - [ ] Set up code coverage reporting
   - [ ] Configure dependabot

5. **Testing**
   - [ ] Achieve 70%+ code coverage (MVP goal)
   - [ ] Add integration tests for all features
   - [ ] **Create dgx-pixels example tests**
   - [ ] Stress testing (long-running tests)

**Success Criteria**:
- All public APIs documented
- Comprehensive test coverage
- No known critical bugs
- **dgx-pixels can adopt term-test**
- Clear, helpful error messages
- CI/CD pipeline green

**Estimated Effort**: 2-3 weeks

## Post-MVP Phases (v0.2.0+)

### Phase 7: Enhanced Features

**Goal**: Nice-to-have features for broader ecosystem

**Priority**: P1 (High)

**Dependencies**: Phase 6 (MVP)

**Tasks**:

1. **Mouse Event Support**
   - [ ] Implement mouse event simulation
   - [ ] Add `send_mouse(event)` method
   - [ ] Test mouse click, drag, scroll

2. **Terminal Resize**
   - [ ] Implement `resize(width, height)` method
   - [ ] Send SIGWINCH signal
   - [ ] Test resize handling

3. **expect-test Integration**
   - [ ] Add expect-test feature flag
   - [ ] Implement expect-compatible output
   - [ ] Create expect-test examples

4. **async-std Support**
   - [ ] Add async-std feature flag
   - [ ] Test with async-std runtime

5. **Performance**
   - [ ] Profile harness overhead
   - [ ] Optimize hot paths
   - [ ] Add benchmarks

**Estimated Effort**: 2-3 weeks

### Phase 8: Advanced Features (Future)

**Goal**: Advanced testing capabilities

**Priority**: P2 (Medium)

**Features**:
- Record/replay terminal sessions
- Visual regression testing
- Fuzzing support
- Terminal coverage metrics
- Multi-terminal testing (xterm, kitty, wezterm)
- Remote testing (SSH)
- Performance profiling tools

## Version Milestones

### v0.1.0 - MVP for dgx-pixels ⭐

**Target**: End of Phase 6

**Includes**:
- ✅ Core PTY harness (Phase 1)
- ✅ Event simulation + async (Phase 2)
- ✅ Sixel position tracking (Phase 3)
- ✅ Bevy ECS integration (Phase 4)
- ✅ Snapshots + assertions (Phase 5)
- ✅ Polish + docs (Phase 6)

**Capabilities**:
- Test dgx-pixels Sixel positioning
- Test dgx-pixels screen transitions
- Test dgx-pixels text input
- Test dgx-pixels Bevy systems
- Run in CI/CD headlessly

**Success**: dgx-pixels can adopt and use term-test for integration testing

### v0.2.0 - Enhanced Features

**Includes**:
- ✅ Mouse events
- ✅ Terminal resize
- ✅ expect-test integration
- ✅ async-std support
- ✅ Performance optimization

### v0.3.0 - Cross-Platform

**Focus**: Windows and macOS support, broader testing

### v1.0.0 - Production Ready

**Focus**: Stable API, comprehensive docs, high adoption

## Dependencies (Updated for MVP)

### Core Dependencies

```toml
[dependencies]
portable-pty = "0.8"
vt100 = "0.15"          # or termwiz if vt100 insufficient
thiserror = "1.0"
```

### MVP Dependencies

```toml
[dependencies.tokio]
version = "1.35"
optional = true
features = ["full"]

[dependencies.bevy]
version = "0.14"
optional = true
default-features = false
features = ["bevy_core"]

[dependencies.bevy_ecs]
version = "0.14"
optional = true

[dependencies.bevy_ratatui]
version = "0.7"
optional = true

[dependencies.ratatui]
version = "0.29"  # dgx-pixels version
optional = true

[dependencies.crossterm]
version = "0.27"
optional = true

[dependencies.insta]
version = "1.34"
optional = true

[dependencies.serde]
version = "1.0"
features = ["derive"]
optional = true

[dependencies.serde_json]
version = "1.0"
optional = true
```

### Development Dependencies

```toml
[dev-dependencies]
tokio-test = "0.4"
```

## Feature Flags (Updated for MVP)

```toml
[features]
default = []

# MVP features
async-tokio = ["tokio"]
bevy = ["dep:bevy", "bevy_ecs"]
bevy-ratatui = ["bevy", "dep:bevy_ratatui"]
ratatui-helpers = ["ratatui", "crossterm"]
sixel = []  # Core Sixel support
snapshot-insta = ["insta", "serde", "serde_json"]

# MVP bundle
mvp = [
    "async-tokio",
    "bevy",
    "bevy-ratatui",
    "ratatui-helpers",
    "sixel",
    "snapshot-insta",
]

# Post-MVP features
async-async-std = ["async-std"]
snapshot-expect = ["expect-test"]
sixel-image = ["image"]  # Advanced Sixel decoding

# Full bundle (all features)
full = [
    "mvp",
    "async-async-std",
    "snapshot-expect",
    "sixel-image",
]
```

## Risk Mitigation (Updated)

### Critical Risks for MVP

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| **vt100 lacks Sixel position tracking** | High | High | Research in Phase 1, use termwiz if needed |
| **Bevy headless mode issues** | Medium | High | Prototype early, consult Bevy community |
| **CI/CD timing issues** | Medium | Medium | Robust timeouts, retry logic |
| **Cross-platform PTY differences** | Low | Medium | Focus on Linux for MVP |

## Implementation Priority (Revised)

**For dgx-pixels MVP**:

1. **Phase 1**: Core PTY + Cursor Tracking (P0 - Foundation)
2. **Phase 2**: Events + Tokio Async (P0 - Input Simulation)
3. **Phase 3**: Sixel Position Tracking (P0 - Graphics Testing) ⭐
4. **Phase 4**: Bevy Integration (P0 - ECS Testing) ⭐
5. **Phase 5**: Snapshots + Assertions (P0 - Developer Experience)
6. **Phase 6**: Polish + Docs (P0 - MVP Release)

**Post-MVP**:
7. **Phase 7**: Enhanced Features (P1)
8. **Future**: Advanced Features (P2)

## Timeline Estimate

### MVP (v0.1.0)

- **Phase 1**: 2-3 weeks
- **Phase 2**: 1-2 weeks
- **Phase 3**: 2-3 weeks (includes vt100 validation)
- **Phase 4**: 2-3 weeks (includes Bevy prototyping)
- **Phase 5**: 1-2 weeks
- **Phase 6**: 2-3 weeks

**Total MVP**: 10-16 weeks (2.5-4 months)

**Aggressive Target**: 3 months
**Realistic Target**: 4 months

### Post-MVP

- **Phase 7**: 2-3 weeks
- **Future phases**: Ongoing

## Success Metrics (Updated for MVP)

### Technical (MVP)

- [ ] Test coverage > 70% (MVP goal, 80% for 1.0)
- [ ] Works on Linux headlessly in CI
- [ ] Zero critical bugs for dgx-pixels use cases
- [ ] API is documented
- [ ] Examples cover all MVP features

### Adoption (MVP)

- [ ] dgx-pixels successfully integrates term-test
- [ ] Can test all 8 dgx-pixels screens
- [ ] Detects Sixel positioning bugs
- [ ] Detects Sixel persistence bugs
- [ ] Test execution time < 100ms per test average

### Post-MVP

- [ ] Published on crates.io
- [ ] Listed in Ratatui ecosystem
- [ ] 3+ projects using term-test
- [ ] Community contributions

## dgx-pixels Integration Checklist

### Pre-Integration (During Phase 1-2)

- [ ] term-test can spawn external binaries
- [ ] term-test can send keyboard events
- [ ] term-test has async Tokio support

### Integration Phase (Phase 3-4)

- [ ] term-test tracks Sixel positions
- [ ] term-test integrates with Bevy
- [ ] term-test supports bevy_ratatui

### Testing Phase (Phase 5-6)

- [ ] Write dgx-pixels integration tests
- [ ] Test all 8 screens
- [ ] Test Sixel positioning
- [ ] Test screen transitions
- [ ] Verify bugs are caught

### Release

- [ ] dgx-pixels adopts term-test
- [ ] CI/CD includes integration tests
- [ ] Documentation includes dgx-pixels examples

## Contributing

See `CONTRIBUTING.md` (to be created in Phase 1)

## License

MIT (to be decided - could also consider MIT/Apache-2.0 dual license)

## References

- **dgx-pixels Issue #1**: TUI Integration Testing Framework Requirements
- **dgx-pixels repo**: https://github.com/raibid-labs/dgx-pixels
- **ratatui**: https://github.com/ratatui/ratatui
- **bevy_ratatui**: https://github.com/cxreiff/bevy_ratatui
- **WezTerm**: https://github.com/wez/wezterm
- **Alacritty**: https://github.com/alacritty/alacritty

## Acknowledgments

This library is being built to support the **dgx-pixels** project and addresses real-world TUI testing needs. Special thanks to the dgx-pixels development for providing concrete requirements and use cases.

---

**Document Status**: Updated for dgx-pixels MVP
**Next Phase**: Phase 1 - Core PTY Harness
**MVP Target**: v0.1.0 in 3-4 months
**Primary Use Case**: dgx-pixels integration testing

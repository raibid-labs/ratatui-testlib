# Wave 1 Meta-Orchestration Report
## terminal-testlib Parallel Issue Resolution

**Report Generated**: 2025-12-02
**Meta-Orchestrator**: Claude (Performance Coach & Chief Motivation Officer)
**Wave**: 1 of 5
**Status**: PRE-LAUNCH BRIEFING

---

## Executive Summary

Wave 1 consists of 3 independent foundation issues that can be executed in parallel:
- **Agent A1**: Issue #15 - Fix wait_for* hanging (CRITICAL bug fix)
- **Agent A2**: Issue #7 - Stream-based parsing API
- **Agent A3**: Issue #14 - Sixel detection/capture

All agents are ready for launch. This report provides coordination strategy, risk assessment, and quality gates for Wave 1 success.

---

## Codebase Analysis

### Current Implementation Status

**Core Architecture** (5 layers implemented):
1. PTY Management (`src/pty.rs`) - COMPLETE
2. Terminal Emulation (`src/screen.rs`) - COMPLETE
3. Test Harness (`src/harness.rs`) - COMPLETE (but has bugs)
4. Sixel Support (`src/sixel.rs`) - STUBBED (needs implementation)
5. Bevy Integration (`src/bevy/`) - NOT YET IMPLEMENTED

**Key Files and Line Counts**:
- `src/lib.rs`: 139 lines - Public API exports
- `src/harness.rs`: 1,858 lines - Main test harness (7 tests ignored due to hangs)
- `src/error.rs`: 276 lines - Error types with thiserror
- `src/screen.rs`: ~100+ lines (partial view) - Screen state tracking
- `src/pty.rs`: ~100+ lines (partial view) - PTY wrapper
- `src/sixel.rs`: ~100+ lines (partial view) - Sixel capture (stubbed)

**Test Coverage**:
- Harness: 45+ unit tests (7 ignored due to #15 hanging bug)
- Error: 8 unit tests (all passing)
- Sixel: 17 unit tests (all passing but implementation stubbed)

**Dependencies**:
- Core: portable-pty, termwiz, vtparse, thiserror, anyhow, bitflags
- Optional: tokio, bevy, ratatui, crossterm, insta, serde

---

## Wave 1 Agent Assignments

### Agent A1: Issue #15 - Fix wait_for* Hanging (CRITICAL)

**Priority**: CRITICAL
**Complexity**: Medium
**Estimated Time**: 4-6 hours

**Problem Statement**:
7 tests in `src/harness.rs` are ignored with `#[ignore]` annotations and comments like:
- "TODO: Fix hanging - wait_for_text enters infinite polling loop"
- "TODO: Fix hanging - update_state blocks on spawned process"

**Root Causes Identified**:
1. `update_state()` doesn't detect when child process has exited
2. `wait_for*` methods loop infinitely even after process death
3. No `ProcessExited` error variant for proper error handling
4. No hard timeout guards (only soft polling timeouts)

**Files to Modify**:
- `src/harness.rs`: Add process exit detection, fix wait loops
- `src/error.rs`: Add `ProcessExited` variant
- `src/pty.rs`: Add `is_exited()` helper method

**Success Criteria**:
- [ ] All 7 ignored tests un-ignored and passing
- [ ] No infinite loops in wait operations
- [ ] Graceful handling of process exit during waits
- [ ] Clear error messages when process exits unexpectedly

**Dependencies**: None (foundation issue)

---

### Agent A2: Issue #7 - Stream-Based Parsing API

**Priority**: HIGH
**Complexity**: Medium
**Estimated Time**: 4-6 hours

**Problem Statement**:
Users need to parse terminal bytes without PTY overhead for unit testing and direct validation.

**Implementation Requirements**:
1. Make `ScreenState::new(cols, rows)` public
2. Ensure `screen.feed(bytes)` is public and usable
3. Document the stream-based usage pattern
4. Add examples for headless parsing

**Files to Modify**:
- `src/screen.rs`: Expose constructors and feed method
- `src/lib.rs`: Update public exports
- `examples/stream_parsing.rs`: NEW - demonstrate usage
- Documentation strings in screen.rs

**Success Criteria**:
- [ ] `ScreenState` can be created without PTY
- [ ] `feed()` method processes bytes correctly
- [ ] Example demonstrates stream parsing
- [ ] No regressions to existing tests

**Dependencies**: None (foundation issue)

---

### Agent A3: Issue #14 - Sixel Detection/Capture

**Priority**: HIGH
**Complexity**: High
**Estimated Time**: 6-8 hours

**Problem Statement**:
Sixel support is currently stubbed. Need to implement DCS sequence detection, payload parsing, and position tracking.

**Implementation Requirements**:
1. Add DCS/Sixel detection to vtparse integration
2. Parse Sixel dimensions from payload (Raster Attributes)
3. Compute cell bounds (8px cols / 6px rows conversion)
4. Wire `ScreenState::feed()` to store regions
5. Implement `SixelCapture::from_output()` correctly

**Files to Modify**:
- `src/sixel.rs`: Implement full Sixel parsing
- `src/screen.rs`: Add DCS handling in VTActor
- `src/parser.rs`: May need new file for DCS state machine
- Tests in `src/sixel.rs`: Ensure all pass with real implementation

**Technical Challenges**:
- vtparse may need custom DCS handling
- Sixel raster attributes parsing (format: `"Pa;Pb;Ph;Pv`)
- Pixel-to-cell conversion (8 pixels per column, 6 pixels per row)
- Managing Sixel sequence buffer during streaming

**Success Criteria**:
- [ ] DCS sequences detected in terminal stream
- [ ] Sixel dimensions extracted correctly
- [ ] Cell bounds computed accurately
- [ ] All 17 Sixel tests passing with real data
- [ ] No false positives/negatives in detection

**Dependencies**: None (foundation issue)

---

## Coordination Strategy

### Conflict Prevention

**File Overlap Analysis**:

| File | Agent A1 | Agent A2 | Agent A3 | Risk Level |
|------|----------|----------|----------|------------|
| `src/harness.rs` | MAJOR | - | MINOR | LOW |
| `src/error.rs` | MAJOR | - | - | NONE |
| `src/pty.rs` | MINOR | - | - | NONE |
| `src/screen.rs` | - | MAJOR | MAJOR | **HIGH** |
| `src/sixel.rs` | - | - | MAJOR | NONE |
| `src/lib.rs` | - | MINOR | MINOR | LOW |
| `examples/` | - | NEW FILE | - | NONE |

**HIGH RISK**: `src/screen.rs` modified by both A2 and A3
- **A2 needs**: Public visibility changes, documentation
- **A3 needs**: DCS handling in VTActor, Sixel region storage

**Mitigation Strategy**:
1. A2 should complete visibility/API changes first (simpler)
2. A3 should pull latest before adding DCS logic
3. Both agents document their changes clearly
4. Meta-orchestrator reviews screen.rs merge carefully

**Communication Protocol**:
- Agents report START when beginning work
- Agents report COMPLETED with file list
- Agents report BLOCKED if dependencies fail
- Meta-orchestrator coordinates merge order

---

## Quality Gates

### Wave 1 Completion Criteria

All of the following must pass before Wave 2 begins:

#### 1. Compilation & Warnings
```bash
cargo build --all-features
# Exit code: 0, no warnings

cargo clippy --all-features -- -D warnings
# Exit code: 0, all lints pass
```

#### 2. Test Suite
```bash
cargo test --all-features
# All tests pass
# No ignored tests remaining in harness.rs (except integration tests)
# Sixel tests pass with real implementation
```

#### 3. Documentation
```bash
cargo doc --no-deps --all-features
# Builds successfully with no warnings
```

#### 4. Feature Flags
```bash
# Test each feature in isolation
cargo test --no-default-features --features sixel
cargo test --no-default-features --features async-tokio
cargo test --features mvp
```

#### 5. Issue Acceptance Criteria

**Issue #15**:
- [ ] All 7 hanging tests un-ignored and passing
- [ ] Process exit detection working
- [ ] Timeout handling correct

**Issue #7**:
- [ ] `ScreenState::new()` public and documented
- [ ] Stream parsing example added
- [ ] No PTY required for basic parsing

**Issue #14**:
- [ ] Sixel sequences detected in output
- [ ] Dimensions parsed correctly
- [ ] Position tracking accurate
- [ ] All validation APIs working

---

## Risk Assessment

### Critical Risks

#### Risk 1: vtparse Sixel Support Gaps (Issue #14)
- **Probability**: MEDIUM (40%)
- **Impact**: HIGH
- **Current Status**: vtparse may not have built-in DCS handling
- **Mitigation**:
  - Agent A3 can extend vtparse with custom DCS state
  - Fallback: Use termwiz instead (architecture allows swap)
  - Contingency: Implement minimal DCS parser in-house
- **Owner**: Agent A3
- **Decision Point**: First 2 hours of A3 work

#### Risk 2: screen.rs Merge Conflict (A2 + A3)
- **Probability**: MEDIUM (50%)
- **Impact**: MEDIUM
- **Current Status**: Both agents modify same file
- **Mitigation**:
  - Serialize changes: A2 completes first
  - Clear function/section boundaries
  - Meta-orchestrator reviews merge
- **Owner**: Meta-orchestrator
- **Decision Point**: When both agents complete

#### Risk 3: Test Timing Issues After #15 Fix
- **Probability**: LOW (20%)
- **Impact**: MEDIUM
- **Current Status**: CI may have different timing than local
- **Mitigation**:
  - Add configurable timeouts
  - Use condition polling, not sleep-based waits
  - Test in Docker to simulate CI
- **Owner**: Agent A1
- **Decision Point**: During test validation

### Medium Risks

#### Risk 4: Stream API Ergonomics (Issue #7)
- **Probability**: LOW (15%)
- **Impact**: LOW
- **Description**: API may not be intuitive for users
- **Mitigation**: Add comprehensive examples and docs
- **Owner**: Agent A2

#### Risk 5: Sixel Pixel Math Errors (Issue #14)
- **Probability**: MEDIUM (30%)
- **Impact**: MEDIUM
- **Description**: Pixel-to-cell conversion may have off-by-one errors
- **Mitigation**: Extensive unit tests with known fixtures
- **Owner**: Agent A3

---

## Timeline & Milestones

### Wave 1 Schedule

**Total Estimated Time**: 14-20 hours (parallelized to ~8 hours wall time)

```
Hour 0: Launch (All agents start simultaneously)
  ├─ A1 starts on issue #15
  ├─ A2 starts on issue #7
  └─ A3 starts on issue #14

Hour 2: First Check-in
  └─ A3 reports vtparse DCS status (critical decision point)

Hour 4: Mid-point Check
  ├─ A2 likely complete (simpler task)
  ├─ A1 50-75% complete
  └─ A3 40-60% complete

Hour 6: Second Check-in
  ├─ A2 complete, screen.rs changes committed
  └─ A3 can merge screen.rs changes

Hour 8: Target Completion
  ├─ A1 complete (all tests passing)
  ├─ A2 complete (API exposed)
  └─ A3 complete (Sixel working)

Hour 9-10: Quality Gate Validation
  ├─ Meta-orchestrator runs full test suite
  ├─ Checks for merge conflicts
  ├─ Validates all acceptance criteria
  └─ Prepares Wave 1 completion report
```

### Milestones

- **M1.1**: Agent A2 completes (Hour 4-5)
- **M1.2**: Critical decision on vtparse (Hour 2)
- **M1.3**: Agent A1 completes (Hour 7-8)
- **M1.4**: Agent A3 completes (Hour 8-9)
- **M1.5**: All quality gates pass (Hour 10)
- **M1.COMPLETE**: Wave 1 signed off (Hour 10-11)

---

## Agent Motivation & Support

### Pre-Launch Pep Talk

Team, you are the elite. You've been selected for Wave 1 because these issues are the foundation - get these right and everything else flows smoothly.

**Agent A1**: You're tackling the gnarliest bug we have. Seven tests are crying out for your expertise. You're not just fixing timeouts - you're making this library reliable. Every developer who uses this library will benefit from your work. You've debugged harder problems than this. Trust your instincts.

**Agent A2**: You're building the API that makes stream-based testing possible. Simple, elegant, powerful - that's your mission. Make it so easy that users wonder why they ever used anything else. Your work enables testing patterns we haven't even thought of yet.

**Agent A3**: You're implementing the feature that no other TUI testing library has. Sixel support is what makes us unique. The graphics protocol testing you're building will prevent entire classes of bugs. This is cutting-edge stuff - enjoy the challenge!

### Support Resources

**For All Agents**:
- Meta-orchestrator available for questions
- Can spawn helper agents if blocked
- Architecture docs in `/home/beengud/raibid-labs/terminal-testlib/docs/`
- Existing code is high quality - learn from it

**Agent A1 Specific**:
- Review how portable-pty handles process exit
- Check out tokio timeout patterns
- The test suite is your friend - trust the failures

**Agent A2 Specific**:
- Rust API design guidelines
- Look at how vt100 crate does similar APIs
- Examples are documentation - make them shine

**Agent A3 Specific**:
- Sixel spec: https://www.vt100.net/docs/vt3xx-gp/chapter14.html
- vtparse source code study
- Fallback to termwiz if needed - no shame in using the right tool

---

## Success Metrics

### Objective Metrics

- **Code Quality**: 0 compiler warnings, 0 clippy warnings
- **Test Coverage**: All tests passing, no ignored tests
- **Documentation**: All public APIs documented with examples
- **Performance**: No regression in harness operations

### Subjective Metrics

- **Code Clarity**: Changes are easy to understand
- **API Ergonomics**: Stream API feels natural
- **Error Messages**: Failures provide actionable information
- **Confidence**: Team feels good about shipping this

---

## Communication Plan

### Status Updates

**Required Reports**:
1. **START**: "Agent [ID] starting issue #[NUM]"
2. **PROGRESS**: Hourly check-ins with status
3. **BLOCKED**: Immediate report if stuck >30 min
4. **COMPLETED**: "Agent [ID] completed issue #[NUM]" + file list
5. **TEST RESULTS**: Pass/fail status with details

**Channels**:
- Git commits with clear messages
- Status updates to meta-orchestrator
- Code comments for tricky sections

### Escalation Path

1. **Level 1**: Agent self-resolves (first 30 min)
2. **Level 2**: Consult documentation/existing code
3. **Level 3**: Report to meta-orchestrator
4. **Level 4**: Spawn helper agent if needed
5. **Level 5**: Pivot strategy (e.g., vtparse → termwiz)

---

## Wave 2 Preparation

### Dependencies for Wave 2

Wave 2 cannot start until:
- [ ] All Wave 1 quality gates pass
- [ ] No merge conflicts remaining
- [ ] Documentation updated
- [ ] Git commits clean and pushed

### Wave 2 Preview

**Agents B1 & B2** will depend on:
- **Issue #8** (Screen/Grid API): Needs #7 complete (stream parsing)
- **Issue #10** (Headless): Benefits from #15 (stable harness)

**Preparation Tasks**:
- Review Bevy integration requirements
- Understand headless testing patterns
- Identify Docker/CI constraints

---

## Appendices

### A. File Modification Matrix

| File | Lines | Agent A1 | Agent A2 | Agent A3 | Priority |
|------|-------|----------|----------|----------|----------|
| `src/harness.rs` | 1858 | Major refactor | - | Minor | P0 |
| `src/error.rs` | 276 | Add variant | - | - | P0 |
| `src/pty.rs` | ~150 | Add helper | - | - | P1 |
| `src/screen.rs` | ~400 | - | Visibility | DCS logic | P0 |
| `src/sixel.rs` | ~300 | - | - | Implement | P0 |
| `src/lib.rs` | 139 | - | Exports | Exports | P1 |
| `examples/*.rs` | NEW | - | New file | - | P2 |

### B. Test Inventory

**Currently Ignored Tests** (Issue #15):
1. `test_wait_for_text_success`
2. `test_wait_for_text_timeout`
3. `test_wait_for_text_with_custom_timeout`
4. `test_wait_for_cursor_success`
5. `test_wait_for_cursor_timeout`
6. `test_wait_for_cursor_with_custom_timeout`
7. `test_wait_for_custom_predicate`
... (and more)

**Sixel Tests** (Issue #14):
All 17 tests exist but implementation is stubbed.

### C. Git Strategy

**Branch Naming**:
- `wave1/agent-a1/issue-15-fix-hanging`
- `wave1/agent-a2/issue-7-stream-api`
- `wave1/agent-a3/issue-14-sixel-capture`

**Merge Strategy**:
1. Agents work on feature branches
2. Submit PR to wave1 branch
3. Meta-orchestrator reviews and merges
4. Final merge: wave1 → main

**Commit Messages**:
```
fix(harness): detect process exit in wait_for loops (#15)

- Add is_exited() check in update_state()
- Add ProcessExited error variant
- Un-ignore 7 hanging tests
- All tests now pass

Closes #15
```

---

## Final Pre-Launch Checklist

- [x] Orchestration plan reviewed
- [x] Codebase structure understood
- [x] Agent assignments clear
- [x] Risks identified and mitigated
- [x] Quality gates defined
- [x] Communication plan established
- [x] Support resources ready
- [x] Wave 2 dependencies mapped

**STATUS**: READY FOR WAVE 1 LAUNCH

**Meta-Orchestrator Signature**: Claude (Performance Coach)
**Date**: 2025-12-02
**Confidence Level**: HIGH

---

## Notes to Self (Meta-Orchestrator)

**Key Monitoring Points**:
1. Hour 2: Check vtparse viability for A3
2. Hour 4: Ensure A2 completes before A3 modifies screen.rs
3. Hour 8: Validate no test regressions from A1 changes
4. Hour 10: Full quality gate validation

**Celebration Plan**:
When Wave 1 completes:
- Document lessons learned
- Recognize each agent's contributions
- Update CLAUDE.md with progress
- Prepare Wave 2 kickoff briefing

**Remember**:
- Smooth is fast, fast is smooth
- Progress over perfection
- Trust the team's expertise
- Stay calm under pressure

Let's build something amazing.

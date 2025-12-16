# Parallel Orchestration Plan for terminal-testlib

## Issue Dependency Graph

```
                    ┌─────────────────────────────────────────────┐
                    │          WAVE 1 (Foundation)                │
                    │   No dependencies - Execute in parallel     │
                    └─────────────────────────────────────────────┘
                              │         │         │
                    ┌─────────┼─────────┼─────────┼─────────┐
                    ▼         ▼         ▼                   │
              ┌─────────┐ ┌─────────┐ ┌─────────┐           │
              │  #15    │ │   #7    │ │  #14    │           │
              │ BUG FIX │ │ Stream  │ │ Sixel   │           │
              │ wait_*  │ │ Parsing │ │ Capture │           │
              └────┬────┘ └────┬────┘ └────┬────┘           │
                   │           │           │                │
                    ┌──────────┴───────────┘                │
                    │                                       │
                    ▼                                       │
         ┌─────────────────────────────────────────────┐    │
         │           WAVE 2 (Core Features)            │    │
         │       Depends on Wave 1 completion          │    │
         └─────────────────────────────────────────────┘    │
                    │                   │                   │
              ┌─────┴─────┐       ┌─────┴─────┐             │
              ▼           ▼       ▼           ▼             │
        ┌─────────┐ ┌─────────┐                             │
        │   #8    │ │  #10    │◄────────────────────────────┘
        │ Screen/ │ │Headless │
        │  Grid   │ │ Support │
        └────┬────┘ └────┬────┘
             │           │
             └─────┬─────┘
                   │
                   ▼
         ┌─────────────────────────────────────────────┐
         │         WAVE 3 (Integration)                │
         │     Depends on Wave 2 completion            │
         └─────────────────────────────────────────────┘
                    │                   │
              ┌─────┴─────┐       ┌─────┴─────┐
              ▼           ▼       ▼           ▼
        ┌─────────┐ ┌─────────┐
        │   #9    │ │  #11    │
        │  Bevy   │ │Position │
        │  ECS    │ │ Assert  │
        └────┬────┘ └────┬────┘
             │           │
             └─────┬─────┘
                   │
                   ▼
         ┌─────────────────────────────────────────────┐
         │         WAVE 4 (Advanced Features)          │
         │     Depends on Wave 3 completion            │
         └─────────────────────────────────────────────┘
                    │                   │
              ┌─────┴─────┐       ┌─────┴─────┐
              ▼           ▼       ▼           ▼
        ┌─────────┐ ┌─────────┐
        │  #16    │ │  #12    │
        │Schedule │ │Snapshot │
        │ Runner  │ │  Bevy   │
        └─────────┘ └─────────┘
                   │
                   ▼
         ┌─────────────────────────────────────────────┐
         │           WAVE 5 (Polish)                   │
         │         Low priority, can run anytime       │
         └─────────────────────────────────────────────┘
                         │
                   ┌─────┴─────┐
                   ▼           ▼
             ┌─────────┐
             │  #13    │
             │  Perf   │
             │Benchmark│
             └─────────┘
```

## Wave Breakdown

### WAVE 1: Foundation (Parallel - 3 agents)

These issues have NO dependencies and can be executed simultaneously.

| Agent | Issue | Title | Priority | Estimated Complexity |
|-------|-------|-------|----------|---------------------|
| A1 | #15 | Fix wait_for* hanging and update_state blocking | CRITICAL | Medium |
| A2 | #7 | Add public API for headless/stream-based parsing | HIGH | Medium |
| A3 | #14 | Implement Sixel detection/capture (stubbed APIs) | HIGH | High |

**Sync Point:** All Wave 1 agents must complete before Wave 2 begins.

**Wave 1 Details:**

#### Agent A1: Bug Fix (#15)
- Make `update_state` non-blocking
- Add `ProcessExited` error variant
- Add hard timeout guards to `wait_for*`
- Un-ignore hanging tests
- **Files:** `src/harness.rs`, `src/error.rs`

#### Agent A2: Stream Parsing API (#7)
- Create `ScreenState::new(cols, rows)`
- Implement `parser.process(&mut screen, bytes)`
- No PTY overhead for direct byte processing
- **Files:** `src/screen.rs`, `src/parser.rs`, `src/lib.rs`

#### Agent A3: Sixel Implementation (#14)
- Add DCS/Sixel detection to vtparse
- Parse Sixel payloads for dimensions
- Compute cell bounds (8px cols / 6px rows)
- Wire `ScreenState::feed` for region storage
- Implement `SixelCapture::from_output`
- **Files:** `src/sixel.rs`, `src/screen.rs`, `src/parser.rs`

---

### WAVE 2: Core Features (Parallel - 2 agents)

Depends on: Wave 1 completion (especially #7 and #15)

| Agent | Issue | Title | Priority | Depends On |
|-------|-------|-------|----------|------------|
| B1 | #8 | Expose Screen/Grid state for verification | HIGH | #7 |
| B2 | #10 | Support headless testing without display server | HIGH | None (but benefits from #15) |

**Sync Point:** All Wave 2 agents must complete before Wave 3 begins.

**Wave 2 Details:**

#### Agent B1: Screen State API (#8)
- Add `screen.get_cell(col, row)` accessor
- Expose `Cell { char, fg, bg }` fields
- Add `screen.snapshot()` for structured export
- Iteration API for rows/cells
- **Files:** `src/screen.rs`, `src/cell.rs`, `src/lib.rs`

#### Agent B2: Headless Support (#10)
- Mock display-dependent operations
- Bevy `MinimalPlugins` integration
- Add `--headless` feature flag
- Docker/CI compatibility
- **Files:** `src/bevy/mod.rs`, `Cargo.toml`, tests

---

### WAVE 3: Integration (Parallel - 2 agents)

Depends on: Wave 2 completion

| Agent | Issue | Title | Priority | Depends On |
|-------|-------|-------|----------|------------|
| C1 | #9 | Add Bevy ECS integration | HIGH | #7, #8, #10 |
| C2 | #11 | Add assertions for UI positioning/layout | MEDIUM | #8 |

**Sync Point:** All Wave 3 agents must complete before Wave 4 begins.

**Wave 3 Details:**

#### Agent C1: Bevy ECS Integration (#9)
- Create `BevyTuiTestHarness` struct
- Add `query<T: Component>()` method
- Add `assert_component_exists<T>()`
- Bridge PTY testing with ECS querying
- **Files:** `src/bevy/harness.rs`, `src/bevy/mod.rs`

#### Agent C2: Position Assertions (#11)
- `assert_within_bounds(component, bounds)`
- `assert_at_position(component, x, y)`
- `assert_no_overlap(c1, c2)`
- `assert_aligned(c1, c2, axis)`
- **Files:** `src/assertions/position.rs`, `src/assertions/mod.rs`

---

### WAVE 4: Advanced Features (Parallel - 2 agents)

Depends on: Wave 3 completion

| Agent | Issue | Title | Priority | Depends On |
|-------|-------|-------|----------|------------|
| D1 | #16 | Add headless ScheduleRunner + GH Actions | HIGH | #9, #10 |
| D2 | #12 | Snapshot testing for Bevy ECS components | MEDIUM | #9 |

**Wave 4 Details:**

#### Agent D1: ScheduleRunner + CI (#16)
- Create `HeadlessBevyRunner` (feature-gated)
- `MinimalPlugins + ScheduleRunnerPlugin` setup
- bevy_ratatui adapter for Frame capture
- insta/expect-test helper integration
- GitHub Actions workflow example
- **Files:** `src/bevy/runner.rs`, `.github/workflows/ci.yml`, docs

#### Agent D2: Bevy Snapshot Testing (#12)
- `ComponentSnapshot` struct with serialization
- `snapshot_components<T: Component>()` method
- insta integration for JSON snapshots
- **Files:** `src/bevy/snapshot.rs`, `src/bevy/mod.rs`

---

### WAVE 5: Polish (Single agent - Low priority)

Can run independently after any wave.

| Agent | Issue | Title | Priority | Depends On |
|-------|-------|-------|----------|------------|
| E1 | #13 | Add performance profiling/benchmarking | LOW | None (nice-to-have) |

#### Agent E1: Performance Utilities (#13)
- `benchmark_rendering(iterations)` method
- `profile_update_cycle()` method
- `assert_fps(min_fps)` assertion
- `BenchmarkResults` with p50/p95/p99
- **Files:** `src/bench/mod.rs`, benches/

---

## Meta-Orchestrator Responsibilities

The meta-orchestrator supervises all waves and ensures:

1. **Wave Synchronization**: No wave starts until previous wave completes
2. **Conflict Resolution**: Merge conflicts between parallel agents
3. **Progress Tracking**: Monitor agent completion status
4. **Quality Gates**: Run tests after each wave
5. **Issue Updates**: Update GitHub issues with progress
6. **Final Integration**: Merge all work and verify CI passes

## Execution Timeline

```
Time ─────────────────────────────────────────────────────────────►

Wave 1:  [====A1====][====A2====][====A3====]
                                            │
                                     ◄──────┴────── Sync Point
                                            │
Wave 2:                                     [====B1====][====B2====]
                                                                   │
                                                            ◄──────┴────── Sync Point
                                                                   │
Wave 3:                                                            [====C1====][====C2====]
                                                                                          │
                                                                                   ◄──────┴────── Sync Point
                                                                                          │
Wave 4:                                                                                   [====D1====][====D2====]
                                                                                                                 │
                                                                                                          ◄──────┴────── Sync Point
                                                                                                                 │
Wave 5:                                                                                                          [====E1====]
```

## Success Criteria

Each wave must pass these criteria before the next begins:

1. **All tests pass**: `cargo test --all-features`
2. **No compiler warnings**: `cargo clippy -- -D warnings`
3. **Docs build**: `cargo doc --no-deps`
4. **Feature flags work**: Test each feature in isolation
5. **CI green**: GitHub Actions workflow passes

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Merge conflicts | Meta-orchestrator resolves, agents work in separate modules |
| Agent blocked | Can escalate to meta-orchestrator, spawn helper agent |
| Test failures | Wave doesn't advance until tests pass |
| Scope creep | Each agent has strict issue-scoped objectives |
| vt100 Sixel gaps | Agent A3 can fall back to termwiz if needed |

## Agent Communication Protocol

Agents report to meta-orchestrator via:
1. **Status updates**: Started, In Progress, Blocked, Completed
2. **Artifacts**: List of files modified, tests added
3. **Blockers**: Dependencies that couldn't be resolved
4. **Test results**: Pass/fail with details

## Post-Completion

After all waves complete:
1. Update CLAUDE.md with implementation notes
2. Update ROADMAP.md with completed phases
3. Create GitHub release (if version bump warranted)
4. Close all issues with commit references
5. Update README.md with new features

---

**Plan Created**: 2025-12-02
**Total Issues**: 10 (excluding meta issue #1)
**Parallel Waves**: 5
**Max Concurrent Agents**: 3 (Wave 1)

# CI/CD Architecture

## Visual Pipeline Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                         GitHub Trigger                          │
│                 (Push, PR, Tag, Manual)                         │
└────────────────────────────┬────────────────────────────────────┘
                             │
                   ┌─────────▼─────────┐
                   │   Event Router    │
                   └─────────┬─────────┘
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
        │                    │                    │
┌───────▼────────┐  ┌────────▼────────┐  ┌───────▼────────┐
│   CI Pipeline  │  │   Release Flow  │  │   Docs Build   │
│   (ci.yml)     │  │  (release.yml)  │  │   (docs.yml)   │
└───────┬────────┘  └────────┬────────┘  └───────┬────────┘
        │                    │                    │
        │                    │                    │
┌───────▼──────────────────────────────────────────────────────┐
│                                                               │
│                    MAIN CI PIPELINE                           │
│                                                               │
│  Stage 1: Quick Check (2-3 min)                              │
│  ┌──────────────────────────────────────────────────────┐    │
│  │  • cargo fmt --check                                 │    │
│  │  • cargo clippy -- -D warnings                       │    │
│  │  Cache: Registry index, dependencies                │    │
│  └────────────────┬─────────────────────────────────────┘    │
│                   │ PASS ✓                                   │
│                   │                                          │
│  Stage 2: Parallel Testing                                   │
│  ┌────────────────┴─────────────────────────────────┐        │
│  │                                                   │        │
│  ├──────────────────────────┬────────────────────────┤        │
│  │                          │                        │        │
│  │  Test Suite             │  Feature Flag Tests     │        │
│  │  (5-7 min)              │  (15-20 min)           │        │
│  │  ┌──────────────────┐   │  ┌──────────────────┐  │        │
│  │  │ Matrix: Rust     │   │  │ Matrix: Features │  │        │
│  │  │ - stable         │   │  │ - no-default     │  │        │
│  │  │ - beta           │   │  │ - all-features   │  │        │
│  │  │                  │   │  │ - async-tokio    │  │        │
│  │  │ Tests:           │   │  │ - bevy           │  │        │
│  │  │ - cargo test     │   │  │ - bevy-ratatui   │  │        │
│  │  │   --lib          │   │  │ - ratatui-help   │  │        │
│  │  │ - cargo test     │   │  │ - sixel          │  │        │
│  │  │   --test '*'     │   │  │ - snapshot-insta │  │        │
│  │  │ - cargo test     │   │  │                  │  │        │
│  │  │   --doc          │   │  │ cargo test       │  │        │
│  │  └──────────────────┘   │  │ <features>       │  │        │
│  │                          │  └──────────────────┘  │        │
│  └──────────┬───────────────┴────────┬───────────────┘        │
│             │ PASS ✓                │ PASS ✓                 │
│             │                        │                        │
│  Stage 3: Quality & Security (parallel)                       │
│  ┌──────────┴────────────────────────┴────────────┐           │
│  │                                                 │           │
│  ├──────────┬──────────┬──────────┬───────────────┤           │
│  │          │          │          │               │           │
│  │ Coverage │ Security │ Examples │ MSRV          │           │
│  │ (8-10min)│ (1-2min) │ (3-5min) │ (3-5min)      │           │
│  │          │          │          │               │           │
│  │ tarpaulin│ cargo-   │ cargo    │ Rust 1.70     │           │
│  │ --all-   │ audit    │ build    │ cargo check   │           │
│  │ features │          │ --examples│ --all-        │           │
│  │          │          │          │ features      │           │
│  │ Upload   │ RustSec  │          │               │           │
│  │ Codecov  │ DB       │          │               │           │
│  └────┬─────┴────┬─────┴────┬─────┴──────┬────────┘           │
│       │ PASS ✓   │ PASS ✓   │ PASS ✓     │ PASS ✓            │
│       │          │          │            │                   │
│  Stage 4: Final Check                                         │
│  ┌────┴──────────┴──────────┴────────────┴──────┐             │
│  │                                               │             │
│  │         CI Success Verification               │             │
│  │         (all jobs must pass)                  │             │
│  │                                               │             │
│  └───────────────────────┬───────────────────────┘             │
│                          │                                     │
└──────────────────────────┼─────────────────────────────────────┘
                           │
                    ┌──────▼──────┐
                    │  CI PASSED  │
                    └──────┬──────┘
                           │
            ┌──────────────┼──────────────┐
            │              │              │
    ┌───────▼──────┐  ┌────▼────┐  ┌─────▼─────┐
    │ PR Approved  │  │ Merge   │  │  Deploy   │
    │ (if PR)      │  │ to Main │  │ (if tag)  │
    └──────────────┘  └─────────┘  └───────────┘
```

## Release Pipeline

```
┌─────────────────────────────────────────┐
│     git tag -a v0.1.0 -m "..."         │
│     git push origin v0.1.0             │
└──────────────────┬──────────────────────┘
                   │
          ┌────────▼────────┐
          │  Release Trigger │
          │  (release.yml)  │
          └────────┬────────┘
                   │
       ┌───────────┼───────────┐
       │           │           │
┌──────▼──────┐ ┌──▼────────┐ ┌▼──────────┐
│   Create    │ │  Publish  │ │   Build   │
│   GitHub    │ │  Crate    │ │   Docs    │
│   Release   │ │           │ │           │
│             │ │ crates.io │ │ GitHub    │
│ - Extract   │ │           │ │ Pages     │
│   changelog │ │ - Verify  │ │           │
│ - Generate  │ │   version │ │ cargo doc │
│   notes     │ │ - cargo   │ │ --all-    │
│ - Upload    │ │   publish │ │ features  │
│   artifacts │ │           │ │           │
└─────────────┘ └───────────┘ └───────────┘
```

## Dependabot Workflow

```
┌────────────────────────────────┐
│   Monday 09:00 (Weekly)        │
└────────────┬───────────────────┘
             │
    ┌────────▼────────┐
    │   Dependabot    │
    │   Scan          │
    └────────┬────────┘
             │
    ┌────────▼────────────────────┐
    │  Check for Updates          │
    │  - GitHub Actions           │
    │  - Cargo dependencies       │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Group by Ecosystem         │
    │  - tokio* → tokio group     │
    │  - bevy* → bevy group       │
    │  - ratatui* → ratatui group │
    │  - dev deps → dev group     │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Create PRs (max 10)        │
    │  Label: dependencies        │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  CI Pipeline Runs           │
    │  (full test suite)          │
    └────────┬────────────────────┘
             │
        ┌────┴────┐
        │ PASS ✓  │
        └────┬────┘
             │
    ┌────────▼────────────────────┐
    │  Manual Review & Merge      │
    │  (or auto-merge if enabled) │
    └─────────────────────────────┘
```

## Local Development Flow

```
┌─────────────────────────────────┐
│  Developer: Make Changes        │
└────────────┬────────────────────┘
             │
    ┌────────▼────────┐
    │  git add .      │
    └────────┬────────┘
             │
    ┌────────▼────────────────────┐
    │  git commit -m "..."        │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Pre-commit Hook (if inst.) │
    │  - cargo fmt --check        │
    │  - cargo clippy             │
    │  - cargo test --lib         │
    └────────┬────────────────────┘
             │
        ┌────┴────┐
   FAIL │         │ PASS ✓
   ┌────▼────┐    │
   │  Fix &  │    │
   │  Retry  │    │
   └────┬────┘    │
        │         │
        └────┬────┘
             │
    ┌────────▼────────────────────┐
    │  Optional: Run Full Check   │
    │  ./scripts/check-ci.sh      │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  git push origin branch     │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  GitHub CI Pipeline         │
    │  (full suite runs)          │
    └────────┬────────────────────┘
             │
        ┌────┴────┐
   FAIL │         │ PASS ✓
   ┌────▼────┐    │
   │  Fix &  │    │
   │  Push   │    │
   └────┬────┘    │
        │         │
        └────┬────┘
             │
    ┌────────▼────────────────────┐
    │  Create Pull Request        │
    │  (template auto-fills)      │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Code Review                │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Merge to Main              │
    └─────────────────────────────┘
```

## Caching Architecture

```
┌─────────────────────────────────────────┐
│          GitHub Actions Cache            │
└────────────────┬────────────────────────┘
                 │
    ┌────────────┼────────────┐
    │            │            │
┌───▼─────┐  ┌───▼─────┐  ┌──▼──────┐
│Registry │  │Compiled │  │  Git    │
│ Index   │  │Artifacts│  │  Deps   │
│         │  │         │  │         │
│~/.cargo/│  │target/  │  │~/.cargo/│
│registry/│  │         │  │git/db   │
│index    │  │         │  │         │
└───┬─────┘  └───┬─────┘  └──┬──────┘
    │            │            │
    └────────────┼────────────┘
                 │
    ┌────────────▼────────────┐
    │   Cache Key Strategy    │
    │                         │
    │  ${{ runner.os }}-      │
    │  ${{ job }}-            │
    │  cargo-                 │
    │  ${{ hashFiles(         │
    │    '**/Cargo.lock'      │
    │  ) }}                   │
    └────────────┬────────────┘
                 │
    ┌────────────▼────────────┐
    │   Cache Restoration     │
    │   - Exact match first   │
    │   - Prefix match second │
    │   - Miss: Full rebuild  │
    └─────────────────────────┘

Cache Benefits:
- Registry index: Skip network fetch
- Compiled deps: Skip recompilation (biggest win)
- Git deps: Skip git clone/fetch

Hit Rate Target: >80%
Time Saved: ~50-60% per job
Expiration: 7 days of inactivity
```

## Coverage Collection Flow

```
┌─────────────────────────────────┐
│   Coverage Job Triggered        │
└────────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Install cargo-tarpaulin    │
    │  (cached binary)            │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Run Tests with Profiling   │
    │  cargo tarpaulin            │
    │    --all-features           │
    │    --workspace              │
    │    --timeout 300            │
    │    --out xml                │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Collect Coverage Data      │
    │  - Line coverage            │
    │  - Function coverage        │
    │  - Branch coverage (basic)  │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Generate Reports           │
    │  - Cobertura XML            │
    │  - Terminal summary         │
    └────────┬────────────────────┘
             │
       ┌─────┴─────┐
       │           │
┌──────▼──────┐ ┌──▼───────────┐
│   Upload    │ │   Upload     │
│   Codecov   │ │   Artifact   │
│             │ │   (30 days)  │
│ - Dashboard │ │              │
│ - Trends    │ │ - Browse     │
│ - PR comm.  │ │ - Download   │
└─────────────┘ └──────────────┘
```

## Security Scanning Flow

```
┌─────────────────────────────────┐
│   Security Audit Job            │
└────────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Install cargo-audit        │
    │  (cached)                   │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Fetch RustSec DB           │
    │  (Advisory Database)        │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Parse Cargo.lock           │
    │  (all dependencies)         │
    └────────┬────────────────────┘
             │
    ┌────────▼────────────────────┐
    │  Check Each Dependency      │
    │  - Security advisories      │
    │  - Yanked crates            │
    │  - Unmaintained crates      │
    └────────┬────────────────────┘
             │
        ┌────┴────┐
   VULN │         │ CLEAN ✓
   ┌────▼────┐    │
   │  FAIL   │    │
   │  Report │    │
   │  Issue  │    │
   └─────────┘    │
                  │
         ┌────────▼────────┐
         │  CI CONTINUES   │
         └─────────────────┘

Parallel: Dependabot Security Updates
┌─────────────────────────────────┐
│  Dependabot Scans Daily         │
│  - Security advisories          │
│  - Create high-priority PRs     │
│  - Auto-label: security         │
└─────────────────────────────────┘
```

## Component Diagram

```
┌─────────────────────────────────────────────────────────────┐
│                    GitHub Infrastructure                    │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Actions    │  │  Dependabot  │  │   Codecov    │      │
│  │   Workflows  │  │              │  │  Integration │      │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘      │
│         │                 │                 │              │
└─────────┼─────────────────┼─────────────────┼──────────────┘
          │                 │                 │
┌─────────┼─────────────────┼─────────────────┼──────────────┐
│         │  terminal-testlib CI/CD Infrastructure   │              │
│         │                 │                 │              │
│  ┌──────▼──────────┐  ┌───▼────────┐  ┌─────▼──────┐       │
│  │   Workflows     │  │   Config   │  │   Docs     │       │
│  │                 │  │            │  │            │       │
│  │ - ci.yml        │  │ - depend.  │  │ - Maintain │       │
│  │ - release.yml   │  │   yml      │  │ - Quick    │       │
│  │ - benchmark.yml │  │ - .ignore  │  │   Ref      │       │
│  │ - docs.yml      │  │            │  │ - Future   │       │
│  └──────┬──────────┘  └────────────┘  └────────────┘       │
│         │                                                   │
│  ┌──────▼──────────────────────────────────────┐           │
│  │              Templates                      │           │
│  │                                             │           │
│  │  - PR Template                              │           │
│  │  - Bug Report                               │           │
│  │  - Feature Request                          │           │
│  └─────────────────────────────────────────────┘           │
│                                                             │
│  ┌─────────────────────────────────────────────┐           │
│  │           Helper Scripts                    │           │
│  │                                             │           │
│  │  - check-ci.sh     (local CI check)        │           │
│  │  - coverage-local.sh (coverage gen)         │           │
│  │  - install-hooks.sh (git hooks)            │           │
│  └─────────────────────────────────────────────┘           │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Data Flow

```
Developer Push
      │
      ▼
GitHub Webhook
      │
      ▼
GitHub Actions Runner (Ubuntu)
      │
      ├──> Clone Repository
      │
      ├──> Restore Cache
      │    ├─> Registry Index
      │    ├─> Dependencies
      │    └─> Compiled Artifacts
      │
      ├──> Install Toolchain (Rust stable/beta)
      │
      ├──> Run Jobs (parallel)
      │    ├─> Format Check
      │    ├─> Clippy Lint
      │    ├─> Tests
      │    ├─> Coverage
      │    ├─> Security Audit
      │    ├─> Examples Build
      │    └─> MSRV Check
      │
      ├──> Collect Results
      │
      ├──> Update Cache
      │
      ├──> Upload Artifacts
      │    ├─> Coverage to Codecov
      │    ├─> Coverage to Actions
      │    └─> Test Reports
      │
      ▼
Status Report
      ├──> GitHub Checks ✓/✗
      ├──> PR Comments (coverage)
      └──> Notifications (if failed)
```

## Key Design Decisions

### 1. Parallel Execution
- Jobs run in parallel where possible
- Dependencies explicitly defined (needs: check)
- Reduces total pipeline time by ~60%

### 2. Fail Fast
- Quick Check job runs first
- Failures block subsequent jobs
- Saves CI minutes

### 3. Caching Strategy
- Per-job cache keys
- Cargo.lock hash for cache invalidation
- 7-day expiration
- Significant time savings

### 4. Feature Flag Testing
- Matrix strategy for all combinations
- Ensures feature compatibility
- Critical for library users

### 5. Security First
- cargo-audit on every run
- Dependabot automated updates
- Minimal workflow permissions
- Secrets for sensitive operations

### 6. Developer Experience
- Local scripts mirror CI
- Pre-commit hooks (optional)
- Clear documentation
- Helpful templates

---

**Document Version**: 1.0
**Last Updated**: 2025-11-19
**Maintainer**: Raibid Labs

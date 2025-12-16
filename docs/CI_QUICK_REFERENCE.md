# CI/CD Quick Reference

## Quick Commands

### Check CI Status
```bash
# View recent workflow runs
gh run list --workflow=ci.yml --limit 10

# View specific run
gh run view <run-id>

# Watch current run
gh run watch
```

### Re-run Failed Jobs
```bash
# Re-run failed jobs only
gh run rerun <run-id> --failed

# Re-run all jobs
gh run rerun <run-id>
```

### Local Testing Commands

```bash
# Run all checks locally before pushing
./scripts/pre-push.sh  # (to be created)

# Or run manually:
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo test --no-default-features
cargo doc --all-features --no-deps
```

## CI Job Matrix

| Job | Duration | Caching | Purpose |
|-----|----------|---------|---------|
| Quick Check | 2-3 min | Yes | Fast feedback (fmt, clippy) |
| Test Suite | 5-7 min | Yes | Core functionality tests |
| Feature Tests | 15-20 min | Yes | All feature combinations |
| Coverage | 8-10 min | Yes | Code coverage reporting |
| Security Audit | 1-2 min | Partial | Vulnerability scanning |
| Examples | 3-5 min | Yes | Example compilation |
| MSRV | 3-5 min | Yes | Minimum Rust version check |

**Total Pipeline Duration**: ~15-20 minutes (parallel execution)

## Feature Flag Test Matrix

| Feature Flag | Description | CI Test |
|--------------|-------------|---------|
| `--no-default-features` | Minimal build | Yes |
| `--all-features` | Complete build | Yes |
| `async-tokio` | Tokio async support | Yes |
| `bevy` | Bevy ECS integration | Yes |
| `bevy-ratatui` | Bevy + Ratatui | Yes |
| `ratatui-helpers` | Ratatui test helpers | Yes |
| `sixel` | Sixel graphics support | Yes |
| `snapshot-insta` | Snapshot testing | Yes |

## Common CI Failures and Fixes

### Formatting Failure
```bash
# Error: Code not formatted
cargo fmt --all
git add .
git commit --amend --no-edit
```

### Clippy Warnings
```bash
# Error: Clippy warnings found
cargo clippy --all-targets --all-features -- -D warnings
# Fix warnings in code
git add .
git commit -m "fix: Address clippy warnings"
```

### Test Failures
```bash
# Error: Tests failing in CI but pass locally
# Check for timing issues, add timeouts
# Verify headless compatibility
cargo test --all-features -- --nocapture
```

### Coverage Job Timeout
```yaml
# Increase timeout in ci.yml
- name: Generate code coverage
  run: cargo tarpaulin --timeout 600  # Increase from 300
```

### Feature Flag Conflicts
```bash
# Error: Feature combination fails
# Check Cargo.toml feature dependencies
# Add missing feature guards in code
#[cfg(feature = "your-feature")]
```

## Cache Troubleshooting

### Clear Cache
```bash
# In GitHub UI:
# Settings > Actions > Caches > Delete cache

# Or wait 7 days (automatic expiration)
```

### Verify Cache Effectiveness
```bash
# Check cache hit/miss in Actions logs
# Search for "Cache restored successfully" or "Cache not found"
```

## Dependabot Commands

```bash
# List dependabot PRs
gh pr list --label dependencies

# Auto-merge dependabot PR (if checks pass)
gh pr merge <pr-number> --auto --squash

# Bulk operations
gh pr list --label dependencies --json number -q '.[].number' | \
  xargs -I {} gh pr merge {} --auto --squash
```

## Release Workflow

```bash
# 1. Update version in Cargo.toml
# 2. Update CHANGELOG.md
# 3. Commit changes
git commit -am "chore: Bump version to 0.1.0"

# 4. Create and push tag
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0

# 5. CI automatically creates GitHub release and publishes to crates.io
```

## Benchmark Workflow

```bash
# Trigger benchmark workflow
gh workflow run benchmark.yml

# View benchmark results
gh run list --workflow=benchmark.yml
gh run view <run-id>

# Download benchmark artifacts
gh run download <run-id>
```

## Documentation Workflow

```bash
# Trigger docs workflow
gh workflow run docs.yml

# Check for broken links locally
cargo install cargo-deadlinks
cargo doc --all-features --no-deps
cargo deadlinks --dir target/doc
```

## Environment Variables

Set in workflow files or repository secrets:

```yaml
# CI Detection
CARGO_TERM_COLOR: always
RUST_BACKTRACE: 1

# Coverage
CODECOV_TOKEN: ${{ secrets.CODECOV_TOKEN }}

# Release
CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

## Debugging Failed Runs

### Step 1: View Logs
```bash
gh run view <run-id> --log
```

### Step 2: Download Artifacts
```bash
gh run download <run-id>
```

### Step 3: Reproduce Locally
```bash
# Run same commands as CI
cargo test --all-features --verbose

# Or use act for exact CI environment
act -j test
```

### Step 4: Add Debug Logging
```yaml
# In workflow file
- name: Debug environment
  run: |
    rustc --version
    cargo --version
    env | sort
```

## Pre-commit Hooks (Future)

```bash
# Install pre-commit hooks
./scripts/install-hooks.sh

# Hooks will run:
# - cargo fmt
# - cargo clippy
# - cargo test (fast tests only)
```

## CI Badge for README

```markdown
[![CI](https://github.com/raibid-labs/terminal-testlib/workflows/CI/badge.svg)](https://github.com/raibid-labs/terminal-testlib/actions/workflows/ci.yml)
[![Coverage](https://codecov.io/gh/raibid-labs/terminal-testlib/branch/main/graph/badge.svg)](https://codecov.io/gh/raibid-labs/terminal-testlib)
[![Crates.io](https://img.shields.io/crates/v/terminal-testlib.svg)](https://crates.io/crates/terminal-testlib)
[![Documentation](https://docs.rs/terminal-testlib/badge.svg)](https://docs.rs/terminal-testlib)
```

## Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| CI Total Time | <10 min | ~15-20 min | To optimize |
| Cache Hit Rate | >80% | TBD | Monitor |
| Test Coverage | >70% (MVP) | TBD | Track |
| CI Success Rate | >95% | TBD | Monitor |

## Useful Links

- [CI Workflow](../.github/workflows/ci.yml)
- [Dependabot Config](../.github/dependabot.yml)
- [Codecov Dashboard](https://codecov.io/gh/raibid-labs/terminal-testlib)
- [GitHub Actions](https://github.com/raibid-labs/terminal-testlib/actions)

---

**Tip**: Bookmark this page for quick reference during development!

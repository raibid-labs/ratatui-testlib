# CI/CD Maintenance Guide

## Overview

The terminal-testlib project uses GitHub Actions for continuous integration and deployment. This guide covers how to maintain and troubleshoot the CI/CD pipeline.

## CI/CD Architecture

### Workflow Files

Located in `.github/workflows/`:

1. **ci.yml** - Main CI pipeline (runs on every push/PR)
2. **release.yml** - Release automation (runs on version tags)
3. **benchmark.yml** - Performance benchmarking
4. **docs.yml** - Documentation validation

### CI Pipeline Stages

```
┌─────────────┐
│ Quick Check │ (fmt, clippy)
└──────┬──────┘
       │
       ├──────────────────────────────────┐
       │                                  │
┌──────▼──────┐                  ┌────────▼────────┐
│ Test Suite  │                  │ Feature Tests   │
│ (stable/beta)│                  │ (all combos)    │
└──────┬──────┘                  └────────┬────────┘
       │                                  │
       ├──────────┬──────────┬───────────┼────────┐
       │          │          │           │        │
┌──────▼──────┐ ┌▼────────┐ ┌▼────────┐ ┌▼─────┐ ┌▼────────┐
│  Coverage   │ │ Security│ │Examples │ │ MSRV │ │  Docs   │
└─────────────┘ └─────────┘ └─────────┘ └──────┘ └─────────┘
       │          │          │           │        │
       └──────────┴──────────┴───────────┴────────┘
                      │
              ┌───────▼────────┐
              │  CI Success    │
              └────────────────┘
```

## Job Descriptions

### 1. Quick Check (check)

**Purpose**: Fast feedback on formatting and linting issues

**Runs**:
- `cargo fmt --check` - Ensures code is properly formatted
- `cargo clippy -- -D warnings` - Catches common mistakes and anti-patterns

**Caching**: Registry and dependency cache for fast execution

**Failure**: Indicates code style violations or clippy warnings

### 2. Test Suite (test)

**Purpose**: Run comprehensive test suite on multiple Rust versions

**Matrix**: `[stable, beta]`

**Runs**:
- `cargo test --lib` - Library tests
- `cargo test --test '*'` - Integration tests
- `cargo test --doc` - Documentation tests

**Headless Configuration**: Runs on Ubuntu without X11/Wayland display server

**Failure**: Test failures or panics

### 3. Feature Flag Tests (feature-tests)

**Purpose**: Ensure all feature combinations compile and work

**Matrix Tests**:
- `--no-default-features` - Minimal build
- `--all-features` - Full build
- Individual features: `async-tokio`, `bevy`, `bevy-ratatui`, `ratatui-helpers`, `sixel`, `snapshot-insta`

**Failure**: Feature combination incompatibility or missing conditional compilation

### 4. Code Coverage (coverage)

**Purpose**: Track test coverage and upload to Codecov

**Tool**: `cargo-tarpaulin`

**Configuration**:
- Timeout: 300 seconds
- Output: XML (Cobertura format)
- Workspace: All packages

**Artifacts**: Coverage reports uploaded to Codecov and GitHub Actions

**Failure**: Usually indicates tarpaulin installation or execution issues

### 5. Security Audit (security-audit)

**Purpose**: Check for known vulnerabilities in dependencies

**Tool**: `cargo-audit`

**Runs**: On every CI run

**Failure**: Known vulnerabilities in dependencies (check RustSec Advisory Database)

### 6. Build Examples (examples)

**Purpose**: Ensure all examples compile

**Runs**: `cargo build --examples --all-features`

**Failure**: Example compilation errors

### 7. MSRV Check (msrv)

**Purpose**: Verify Minimum Supported Rust Version (1.70)

**Runs**: `cargo check --all-features`

**Failure**: Code uses features from newer Rust versions

## Caching Strategy

### Cache Keys

```yaml
key: ${{ runner.os }}-${{ job }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

### Cached Paths

1. `~/.cargo/registry/index` - Crate registry index
2. `~/.cargo/registry/cache` - Downloaded crate archives
3. `~/.cargo/git/db` - Git dependencies
4. `target/` - Build artifacts

### Cache Benefits

- Reduces CI time from ~10-15 minutes to ~3-5 minutes
- Reduces network bandwidth usage
- Speeds up dependency compilation

## Dependabot Configuration

Located in `.github/dependabot.yml`

### Update Schedule

- **GitHub Actions**: Weekly (Mondays at 09:00)
- **Cargo Dependencies**: Weekly (Mondays at 09:00)

### Grouping Strategy

Dependencies are grouped by ecosystem:
- `tokio` group: All tokio-related crates
- `bevy` group: All bevy-related crates
- `ratatui` group: ratatui and crossterm
- `testing` group: insta, expect-test
- `development-dependencies`: All dev dependencies

### Pull Request Limits

- GitHub Actions: 5 concurrent PRs
- Cargo: 10 concurrent PRs

## Maintenance Tasks

### Weekly Tasks

1. **Review Dependabot PRs**
   ```bash
   # Check for open dependabot PRs
   gh pr list --label dependencies
   ```

2. **Check CI Success Rate**
   ```bash
   # View recent workflow runs
   gh run list --workflow=ci.yml --limit 20
   ```

3. **Review Coverage Trends**
   - Visit Codecov dashboard
   - Ensure coverage stays above 70% (MVP goal)

### Monthly Tasks

1. **Audit Dependencies**
   ```bash
   cargo audit
   cargo outdated
   ```

2. **Review Cache Effectiveness**
   - Check average CI run times
   - Verify cache hit rates in Actions logs

3. **Update MSRV** (if needed)
   - Test on new Rust versions
   - Update MSRV in workflow and Cargo.toml

### Quarterly Tasks

1. **Benchmark Performance**
   ```bash
   cargo criterion --all-features
   ```

2. **Review and Update CI Pipeline**
   - Check for new GitHub Actions versions
   - Evaluate new CI tools and practices

3. **Security Review**
   - Review all dependencies for security advisories
   - Update to latest patch versions

## Troubleshooting

### Common Issues

#### 1. Flaky Tests

**Symptoms**: Tests pass locally but fail in CI

**Causes**:
- Timing issues in headless environment
- Race conditions
- Platform-specific behavior

**Solutions**:
```rust
// Increase timeout for CI
#[cfg(test)]
const TIMEOUT: Duration = if cfg!(ci) {
    Duration::from_secs(10)
} else {
    Duration::from_secs(5)
};
```

#### 2. Cache Invalidation

**Symptoms**: CI runs slower than expected

**Causes**:
- Cargo.lock changed
- Cache eviction
- Cache corruption

**Solutions**:
```bash
# Manually clear cache in GitHub Actions settings
# Or update cache key in workflow file
```

#### 3. Tarpaulin Failures

**Symptoms**: Coverage job fails or times out

**Causes**:
- Tests hanging in tarpaulin
- Memory issues
- Timeout too short

**Solutions**:
```yaml
# Increase timeout
cargo tarpaulin --timeout 600

# Exclude problematic tests
cargo tarpaulin --exclude-files tests/flaky/*
```

#### 4. Feature Flag Conflicts

**Symptoms**: Feature combination tests fail

**Causes**:
- Missing `#[cfg(feature = "...")]` guards
- Feature dependencies not declared
- Incompatible features

**Solutions**:
```toml
# In Cargo.toml, declare feature dependencies
[features]
bevy-ratatui = ["bevy", "ratatui"]
```

### Debug Failed CI Runs

1. **View Logs**
   ```bash
   gh run view <run-id> --log
   ```

2. **Download Artifacts**
   ```bash
   gh run download <run-id>
   ```

3. **Reproduce Locally**
   ```bash
   # Use act to run GitHub Actions locally
   act -j test
   ```

4. **Check Specific Job**
   ```bash
   gh run view <run-id> --job <job-id>
   ```

## Performance Optimization

### Current Benchmarks

- Quick Check: ~2-3 minutes
- Test Suite: ~5-7 minutes per Rust version
- Feature Tests: ~3-5 minutes per combination
- Coverage: ~8-10 minutes
- **Total CI Time**: ~15-20 minutes (with caching)

### Optimization Strategies

1. **Parallel Execution**
   - Jobs run in parallel where possible
   - Test matrix parallelizes across Rust versions

2. **Incremental Compilation**
   - Cached build artifacts reduce rebuild times
   - Use `cargo build --release` only when needed

3. **Selective Testing**
   ```yaml
   # Only run expensive tests on main branch
   if: github.ref == 'refs/heads/main'
   ```

4. **Smart Caching**
   ```yaml
   # Cache by job type for better hit rates
   key: ${{ runner.os }}-${{ matrix.job }}-cargo-${{ hashFiles('**/Cargo.lock') }}
   ```

## Security Best Practices

### Secrets Management

Required secrets in repository settings:
- `CODECOV_TOKEN` - For coverage uploads
- `CARGO_REGISTRY_TOKEN` - For crates.io publishing

### Permissions

Workflows use minimal permissions:
```yaml
permissions:
  contents: read
  pull-requests: write  # For PR comments
```

### Dependency Auditing

Automated via:
1. Dependabot security updates (enabled)
2. `cargo audit` in CI
3. Manual quarterly reviews

## Monitoring and Alerts

### GitHub Actions Notifications

Configure in repository settings:
- Email notifications for failed runs
- Slack/Discord webhooks for CI status

### Codecov Integration

- Coverage decrease threshold: 5%
- Comment on PRs with coverage changes
- Fail CI if coverage drops significantly

### Metrics to Track

1. **CI Success Rate**: Should be >95%
2. **Average CI Duration**: Target <10 minutes
3. **Test Coverage**: MVP goal >70%, target >80%
4. **Cache Hit Rate**: Should be >80%
5. **Dependabot PR Merge Rate**: Should be >90%

## Future Enhancements

### Planned Improvements

1. **Cross-Platform Testing** (Phase 7+)
   - macOS runners
   - Windows runners
   - Multi-platform matrix

2. **Performance Regression Detection**
   - Automated benchmark comparison
   - Alert on >10% performance degradation

3. **Nightly Builds**
   - Test against Rust nightly
   - Early warning for upcoming breaking changes

4. **Deploy Previews**
   - PR preview environments
   - Documentation previews

5. **Advanced Coverage**
   - Branch coverage
   - Line-by-line coverage in PR comments

6. **Canary Deployments**
   - Gradual rollout to crates.io
   - Beta channel for early adopters

## FAQ

### Q: Why does CI take longer than local tests?

**A**: CI runs more comprehensive checks:
- Multiple Rust versions
- All feature combinations
- Code coverage
- Security audits
- Documentation builds

### Q: How do I skip CI for a commit?

**A**: Add `[skip ci]` to commit message (use sparingly):
```bash
git commit -m "docs: Update README [skip ci]"
```

### Q: Can I run CI locally?

**A**: Yes, using `act`:
```bash
# Install act
brew install act

# Run CI locally
act -j test
```

### Q: How do I update the MSRV?

**A**:
1. Test on new version locally
2. Update `rust-version` in Cargo.toml
3. Update toolchain in `.github/workflows/ci.yml`
4. Document in CHANGELOG.md

### Q: Why are feature tests failing?

**A**: Usually due to:
- Missing feature guards in code
- Undeclared feature dependencies
- Incompatible feature combinations

Check the specific feature matrix job logs for details.

## Resources

- [GitHub Actions Documentation](https://docs.github.com/en/actions)
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin)
- [cargo-audit](https://github.com/RustSec/rustsec/tree/main/cargo-audit)
- [Codecov](https://docs.codecov.com/)
- [Dependabot](https://docs.github.com/en/code-security/dependabot)

## Support

For CI/CD issues:
1. Check this guide
2. Review workflow logs
3. Search existing issues
4. Open a new issue with `[CI]` prefix

---

**Last Updated**: 2025-11-19
**Maintainer**: Raibid Labs
**Status**: Active

# Test Fixtures

This directory contains test fixtures for terminal_testlib integration tests.

## Structure

- `sixel/` - Sixel image test files
  - Reference Sixel files from libsixel and other sources
  - Used for Sixel parsing and position tracking tests (Phase 3)

## Adding Fixtures

When adding new test fixtures:

1. Place files in the appropriate subdirectory
2. Add a comment in the test explaining what the fixture tests
3. Keep file sizes reasonable (< 1MB preferred)
4. Use descriptive filenames

## Phase 3 Fixtures

During Phase 3 (Sixel Support), we will add:

- Sample Sixel images from libsixel (`snake.six`, `map8.six`)
- Sixel images from Jexer test suite
- Custom test images for bounds checking
- Expected rendering snapshots

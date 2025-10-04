# Scripts Directory

This directory contains utility scripts for the wavemark workspace.

## Available Scripts

### `run_all_tests.sh`
Comprehensive test runner that:
- Builds all crates in debug and release modes
- Runs all unit tests
- Runs integration tests for each crate
- Verifies workspace structure
- Provides detailed colored output

**Usage:**
```bash
./scripts/run_all_tests.sh
```

### `quick_test.sh`
Simplified test runner that:
- Builds all crates
- Runs all tests
- Provides minimal output

**Usage:**
```bash
./scripts/quick_test.sh
```

## Running Scripts

Make sure scripts are executable:
```bash
chmod +x scripts/*.sh
```

Run from the workspace root directory:
```bash
# Full test suite
./scripts/run_all_tests.sh

# Quick test
./scripts/quick_test.sh
```

## Script Requirements

- Bash shell
- Rust toolchain (cargo, rustc)
- Run from wavemark workspace root directory

## Exit Codes

- `0`: All tests passed
- `1`: Tests failed or script error

## Customization

You can modify these scripts to:
- Add additional test types
- Change output formatting
- Add performance benchmarks
- Include linting checks

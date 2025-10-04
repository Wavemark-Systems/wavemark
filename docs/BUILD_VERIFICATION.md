# Build Verification

This document describes how to verify that the wavemark workspace is correctly configured and all crates build successfully.

## Quick Verification

Run the following commands to verify the workspace:

```bash
# Build all crates in debug mode
cargo build

# Build all crates in release mode
cargo build --release

# Run all tests
cargo test

# Run tests for a specific component
cargo test -p wavemark
cargo test -p wavemark-python
cargo test -p wavemark-typescript
```

## Expected Output

### Successful Build
```
   Compiling wavemark v0.1.0 (/path/to/wavemark/wavemark)
   Compiling wavemark-python v0.1.0 (/path/to/wavemark/bindings/python)
   Compiling wavemark-typescript v0.1.0 (/path/to/wavemark/bindings/typescript)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in X.XXs
```

### Successful Tests
```
running 2 tests
test tests::test_process_function ... ok
test tests::test_workspace_integration ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## Workspace Structure Verification

The following structure should be present:

```
wavemark/
├── Cargo.toml              # Workspace manifest
├── Cargo.lock              # Dependency lock file
├── README.md               # Main project README
├── LICENSE                 # License file
├── docs/                   # Documentation
│   ├── README.md           # Detailed documentation
│   └── BUILD_VERIFICATION.md # This file
├── scripts/                # Utility scripts
├── bindings/               # Language bindings
│   ├── python/             # Python bindings
│   └── typescript/         # TypeScript bindings
└── wavemark/               # Main library
    ├── Cargo.toml          # Library configuration
    ├── examples/           # Library examples
    ├── tests/              # Library tests
    └── src/                # Source code
        ├── lib.rs          # Main entry point
        ├── encoder/        # Encoder module
        ├── decoder/        # Decoder module
        ├── fourier/        # Fourier module
        └── api/            # API module
```

## Troubleshooting

### Build Failures

If builds fail, check:
1. Rust toolchain is installed: `rustc --version`
2. Cargo is available: `cargo --version`
3. All crate directories exist and contain valid `Cargo.toml` files
4. Workspace manifest is correctly configured

### Test Failures

If tests fail, check:
1. All crates compile successfully first
2. Test functions are properly marked with `#[test]`
3. Dependencies are correctly specified in `Cargo.toml` files

## Continuous Integration

For CI/CD pipelines, use:

```bash
# Install dependencies and build
cargo build --verbose

# Run tests with output
cargo test --verbose

# Check formatting (if rustfmt is configured)
cargo fmt -- --check

# Run clippy lints (if clippy is configured)
cargo clippy -- -D warnings
```

# Wavemark

A Rust workspace for audio watermarking and signal processing.

## Repository Layout

This repository is organized as a Rust workspace containing multiple specialized crates:

```
wavemark/
├── Cargo.toml              # Workspace manifest
├── Cargo.lock              # Dependency lock file
├── LICENSE                  # License file
├── docs/                   # Documentation
│   └── README.md           # This file
├── scripts/                # Utility scripts
│   ├── run_all_tests.sh    # Comprehensive test runner
│   ├── quick_test.sh       # Quick test runner
│   └── README.md           # Scripts documentation
├── bindings/               # Language bindings
│   ├── python/             # Python bindings (PyO3)
│   │   ├── src/            # Rust source for Python bindings
│   │   ├── tests/          # Python tests
│   │   ├── examples/       # Python examples
│   │   └── .cargo/         # Cargo configuration
│   └── typescript/         # TypeScript/WebAssembly bindings
│       ├── src/            # Rust source for WASM bindings
│       ├── tests/          # TypeScript tests
│       ├── examples/       # TypeScript examples
│       └── .cargo/         # Cargo configuration
└── crates/                 # Individual library crates
    ├── encoder/            # Audio encoding functionality
    │   ├── Cargo.toml
    │   ├── src/
    │   │   └── lib.rs
    │   └── tests/
    │       └── build_test.rs
    ├── decoder/            # Audio decoding functionality
    │   ├── Cargo.toml
    │   ├── src/
    │   │   └── lib.rs
    │   └── tests/
    │       └── build_test.rs
    ├── fourier/            # Fourier transform operations
    │   ├── Cargo.toml
    │   ├── src/
    │   │   └── lib.rs
    │   └── tests/
    │       └── build_test.rs
    └── api/                # Main public API
        ├── Cargo.toml
        ├── src/
        │   └── lib.rs
        └── tests/
            └── build_test.rs
```

## Architecture

### Workspace Structure

This project uses Rust's workspace feature to organize related crates and language bindings. The root `Cargo.toml` defines the workspace and lists all member crates and bindings.

### Crate Responsibilities

#### `wavemark-encoder`
- **Purpose**: Handles encoding of audio data into wavemark format
- **Location**: `crates/encoder/`
- **Key Functions**: Audio data transformation, watermark embedding

#### `wavemark-decoder`
- **Purpose**: Handles decoding of wavemark data back to audio format
- **Location**: `crates/decoder/`
- **Key Functions**: Watermark extraction, audio data reconstruction

#### `wavemark-fourier`
- **Purpose**: Provides Fourier transform functionality for signal processing
- **Location**: `crates/fourier/`
- **Key Functions**: FFT operations, frequency domain analysis

#### `wavemark-api`
- **Purpose**: Main public API that orchestrates the other crates
- **Location**: `crates/api/`
- **Key Functions**: High-level operations, user-facing interface

### Language Bindings

#### Python Bindings (`bindings/python/`)
- **Technology**: PyO3 for native Python integration
- **Target Audience**: AI developers, researchers, data scientists
- **Features**: NumPy integration, idiomatic Python API, PyPI distribution
- **Usage**: `pip install wavemark`

#### TypeScript/WebAssembly Bindings (`bindings/typescript/`)
- **Technology**: WebAssembly with wasm-bindgen
- **Target Audience**: Web developers, browser-based applications
- **Features**: Web Audio API integration, near-native performance, NPM distribution
- **Usage**: `npm install wavemark-typescript`

## Development

### Building the Workspace

To build all crates in the workspace:

```bash
cargo build
```

To build a specific crate:

```bash
cargo build -p wavemark-encoder
cargo build -p wavemark-decoder
cargo build -p wavemark-fourier
cargo build -p wavemark-api
```

### Running Tests

To run all tests:

```bash
cargo test
```

To run integration tests specifically:

```bash
cargo test --test integration_test
```

### Adding Dependencies

Dependencies can be added to individual crates by editing their respective `Cargo.toml` files:

```toml
# In crates/encoder/Cargo.toml
[dependencies]
some-crate = "1.0"
```

### Cross-Crate Dependencies

To use one crate from another within the workspace, add it as a dependency:

```toml
# In crates/api/Cargo.toml
[dependencies]
wavemark-encoder = { path = "../encoder" }
wavemark-decoder = { path = "../decoder" }
wavemark-fourier = { path = "../fourier" }
```

## Testing Strategy

### Crate-Level Tests

Each crate contains its own test directory with build verification tests:
- `crates/*/tests/build_test.rs` - Verifies each crate builds and functions correctly
- Unit tests in `src/` files using the `#[cfg(test)]` attribute

### Test Scripts

The `scripts/` directory contains utility scripts for running tests:
- `run_all_tests.sh` - Comprehensive test suite with detailed output
- `quick_test.sh` - Simplified test runner for quick verification

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific crate
cargo test -p wavemark-encoder

# Use test scripts
./scripts/quick_test.sh
./scripts/run_all_tests.sh
```

## Future Development

### Planned Features

1. **Audio Format Support**: Add support for various audio formats (WAV, MP3, etc.)
2. **Watermarking Algorithms**: Implement robust watermarking techniques
3. **Performance Optimization**: Optimize for real-time processing
4. **Error Handling**: Comprehensive error handling and recovery
5. **Documentation**: Detailed API documentation for each crate

### Adding New Crates

To add a new crate to the workspace:

1. Create the crate directory: `mkdir crates/new-crate`
2. Initialize the crate: `cd crates/new-crate && cargo init --lib`
3. Add to workspace: Update root `Cargo.toml` to include the new crate in `members`
4. Update documentation: Add the new crate to this README

## License

This project is licensed under the terms specified in the `LICENSE` file.

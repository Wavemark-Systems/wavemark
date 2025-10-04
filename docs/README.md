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
│   ├── README.md           # This file
│   └── BUILD_VERIFICATION.md # Build verification guide
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
└── wavemark/               # Main audio watermarking library
    ├── Cargo.toml          # Library configuration
    ├── examples/           # Library examples
    │   └── basic_usage.rs  # Basic usage example
    ├── tests/              # Library tests
    │   ├── test_encoder.rs # Encoder tests
    │   ├── test_decoder.rs # Decoder tests
    │   ├── test_fourier.rs # Fourier tests
    │   └── test_api.rs     # API tests
    └── src/                # Source code with modular structure
        ├── lib.rs          # Main library entry point
        ├── encoder/        # Encoder module
        │   └── mod.rs      # Encoder implementation
        ├── decoder/        # Decoder module
        │   └── mod.rs      # Decoder implementation
        ├── fourier/        # Fourier module
        │   └── mod.rs      # Fourier implementation
        └── api/            # API module
            └── mod.rs      # API implementation
```

## Architecture

### Workspace Structure

This project uses Rust's workspace feature to organize a unified library and language bindings. The root `Cargo.toml` defines the workspace and lists the main library and binding members.

### Library Structure

#### `wavemark/` - Main Library
- **Purpose**: Unified audio watermarking library with modular architecture
- **Location**: `wavemark/`
- **Structure**: Single library with organized modules in subdirectories

#### Module Responsibilities

##### `encoder/` Module
- **Purpose**: Handles encoding of audio data into wavemark format
- **Location**: `wavemark/src/encoder/`
- **Key Functions**: Audio data transformation, watermark embedding

##### `decoder/` Module
- **Purpose**: Handles decoding of wavemark data back to audio format
- **Location**: `wavemark/src/decoder/`
- **Key Functions**: Watermark extraction, audio data reconstruction

##### `fourier/` Module
- **Purpose**: Provides Fourier transform functionality for signal processing
- **Location**: `wavemark/src/fourier/`
- **Key Functions**: FFT operations, frequency domain analysis

##### `api/` Module
- **Purpose**: Main public API that orchestrates the other modules
- **Location**: `wavemark/src/api/`
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

To build all components in the workspace:

```bash
cargo build
```

To build specific components:

```bash
# Build main library
cargo build -p wavemark

# Build Python bindings
cargo build -p wavemark-python

# Build TypeScript bindings
cargo build -p wavemark-typescript
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

### Module Dependencies

The modules are organized within a single library, so they can reference each other directly:

```rust
// In wavemark/src/api/mod.rs
use crate::encoder;
use crate::decoder;
use crate::fourier;
```

### Binding Dependencies

Language bindings depend on the main library:

```toml
# In bindings/python/Cargo.toml
[dependencies]
wavemark = { workspace = true }
```

## Testing Strategy

### Library Tests

The main library contains comprehensive tests:
- `wavemark/tests/` - Integration tests for each module
- Unit tests in `src/` files using the `#[cfg(test)]` attribute

### Test Scripts

The `scripts/` directory contains utility scripts for running tests:
- `run_all_tests.sh` - Comprehensive test suite with detailed output
- `quick_test.sh` - Simplified test runner for quick verification

### Running Tests

```bash
# Run all tests
cargo test

# Run tests for specific component
cargo test -p wavemark
cargo test -p wavemark-python
cargo test -p wavemark-typescript

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

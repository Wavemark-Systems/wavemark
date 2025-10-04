# Wavemark

Wavemark is an open-source toolkit for embedding imperceptible, verifiable information within audio signals. It enables creators, researchers, and AI developers to encode provenance, metadata, or cryptographic proofs directly into sound — without affecting audio quality.

## Quick Start

```bash
# Build all crates
cargo build

# Run tests
cargo test

# Run quick test script
./scripts/quick_test.sh

# Run comprehensive test suite
./scripts/run_all_tests.sh
```

## Repository Structure

This is a Rust workspace containing four specialized crates and language bindings:

### Core Crates
- **`wavemark-encoder`** - Audio encoding and watermark embedding
- **`wavemark-decoder`** - Audio decoding and watermark extraction  
- **`wavemark-fourier`** - Fourier transform operations for signal processing
- **`wavemark-api`** - Main public API that orchestrates all functionality

### Language Bindings
- **`bindings/python/`** - Python bindings using PyO3 (for AI developers)
- **`bindings/typescript/`** - TypeScript/WebAssembly bindings (for web applications)

## Documentation

For detailed documentation, see [`docs/README.md`](docs/README.md).

## Development Status

🚧 **Early Development** - This project is in the initial setup phase. Core functionality is being implemented.

## License

See [LICENSE](LICENSE) for license information.

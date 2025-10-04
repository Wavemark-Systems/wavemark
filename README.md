# Wavemark

A Rust workspace for audio watermarking and signal processing.

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

This is a Rust workspace containing four specialized crates:

- **`wavemark-encoder`** - Audio encoding and watermark embedding
- **`wavemark-decoder`** - Audio decoding and watermark extraction  
- **`wavemark-fourier`** - Fourier transform operations for signal processing
- **`wavemark-api`** - Main public API that orchestrates all functionality

## Documentation

For detailed documentation, see [`docs/README.md`](docs/README.md).

## Development Status

🚧 **Early Development** - This project is in the initial setup phase. Core functionality is being implemented.

## License

See [LICENSE](LICENSE) for license information.

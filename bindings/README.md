# Wavemark Language Bindings

This directory contains language bindings for the Wavemark audio watermarking library, making it accessible from Python and TypeScript/JavaScript.

## Structure

```
bindings/
├── python/          # Python bindings using PyO3
│   ├── src/         # Rust source code for Python bindings
│   ├── tests/       # Python tests
│   ├── examples/    # Python usage examples
│   ├── .cargo/      # Cargo configuration
│   ├── Cargo.toml   # Rust project configuration
│   └── pyproject.toml # Python package configuration
└── typescript/      # TypeScript/WebAssembly bindings
    ├── src/         # Rust source code for WASM bindings
    ├── tests/       # TypeScript tests
    ├── examples/    # TypeScript usage examples
    ├── .cargo/      # Cargo configuration
    ├── Cargo.toml   # Rust project configuration
    └── package.json # Node.js package configuration
```

## Python Bindings

The Python bindings provide a native Python interface to Wavemark using PyO3.

### Features
- Native Python classes for encoder, decoder, and Fourier processor
- NumPy array support for audio data
- Standalone functions for simple usage
- Full integration with Python ecosystem

### Installation
```bash
# From source (requires Rust toolchain)
cd bindings/python
pip install maturin
maturin develop

# Or install from PyPI (when published)
pip install wavemark
```

### Usage
```python
import wavemark
import numpy as np

# Create encoder and decoder
encoder = wavemark.WavemarkEncoder()
decoder = wavemark.WavemarkDecoder()

# Encode watermark
audio_data = np.sin(np.linspace(0, 2*np.pi, 1000)).astype(np.float32)
watermarked = encoder.encode(audio_data.tolist(), "my_watermark")

# Decode watermark
extracted = decoder.decode(watermarked)
```

## TypeScript/WebAssembly Bindings

The TypeScript bindings provide WebAssembly-based access to Wavemark for web applications.

### Features
- WebAssembly for near-native performance
- TypeScript type definitions
- Web Audio API integration
- Browser and Node.js support

### Installation
```bash
# For web applications
cd bindings/typescript
npm install
npm run build

# For Node.js applications
npm run build:node
```

### Usage
```typescript
import { WavemarkEncoder, WavemarkDecoder } from './pkg/wavemark_typescript';

// Create encoder and decoder
const encoder = new WavemarkEncoder();
const decoder = new WavemarkDecoder();

// Encode watermark
const audioData = new Float32Array(1000);
const watermarked = encoder.encode(audioData, "my_watermark");

// Decode watermark
const extracted = decoder.decode(watermarked);
```

## Development

### Building All Bindings
```bash
# From workspace root
cargo build

# Build specific binding
cargo build -p wavemark-python
cargo build -p wavemark-typescript
```

### Testing
```bash
# Python tests
cd bindings/python
python -m pytest tests/

# TypeScript tests
cd bindings/typescript
npm test
```

### Examples
Each binding directory contains comprehensive examples:
- `basic_usage.py/ts` - Simple encoding/decoding
- `ai_voice_watermarking.py` - AI voice generation integration
- `web_audio_integration.ts` - Web Audio API integration

## Architecture

Both bindings follow the same architectural pattern:
1. **Rust Core**: Uses the same underlying crates (`wavemark-encoder`, `wavemark-decoder`, etc.)
2. **Language-Specific Wrapper**: Provides idiomatic API for each language
3. **Type Safety**: Full type definitions and error handling
4. **Performance**: Minimal overhead through direct Rust integration

## Contributing

When adding new features:
1. Implement in the Rust core crates first
2. Add bindings to both Python and TypeScript
3. Update examples and tests
4. Update documentation

## License

Same as the main Wavemark project (MIT License).

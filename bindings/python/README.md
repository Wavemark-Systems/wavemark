# Wavemark Python Bindings

Python bindings for the Wavemark audio watermarking library, built with PyO3 for native performance.

## Quick Start

```python
import wavemark
import numpy as np

# Create encoder and decoder
encoder = wavemark.WavemarkEncoder()
decoder = wavemark.WavemarkDecoder()

# Generate sample audio
audio_data = np.sin(np.linspace(0, 2*np.pi, 1000)).astype(np.float32)

# Encode watermark
watermarked = encoder.encode(audio_data.tolist(), "AI_VOICE_v1.0")

# Decode watermark
extracted = decoder.decode(watermarked)
print(f"Extracted watermark: {extracted}")
```

## Installation

### From Source
```bash
# Install maturin
pip install maturin

# Build and install
maturin develop
```

### From PyPI (when published)
```bash
pip install wavemark
```

## API Reference

### WavemarkEncoder
```python
encoder = wavemark.WavemarkEncoder()
watermarked_audio = encoder.encode(audio_data, watermark)
```

### WavemarkDecoder
```python
decoder = wavemark.WavemarkDecoder()
watermark = decoder.decode(audio_data)
is_valid = decoder.verify(audio_data, expected_watermark)
```

### FourierProcessor
```python
processor = wavemark.FourierProcessor()
transformed = processor.transform(audio_data)
```

### Standalone Functions
```python
# Direct function calls
watermarked = wavemark.encode_audio(audio_data, watermark)
watermark = wavemark.decode_audio(audio_data)
```

## Examples

See the `examples/` directory for:
- Basic usage patterns
- AI voice generation integration
- NumPy array handling
- Error handling

## Requirements

- Python 3.8+
- NumPy (for examples)
- Rust toolchain (for building from source)

## Development

```bash
# Run tests
python -m pytest tests/

# Build wheel
maturin build --release

# Install in development mode
maturin develop
```

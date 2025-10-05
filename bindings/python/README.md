# Wavemark Python Bindings

This directory contains Python bindings for the Wavemark audio watermarking library.

## Overview

The Python package `wavemark` provides Rust-powered audio watermarking functionality through PyO3 bindings, built and distributed as pre-compiled wheels via maturin.

## Installation

### From PyPI (when published)
```bash
pip install wavemark
```

### Development Installation
```bash
# Install maturin
pip install maturin

# Build and install in development mode
cd bindings/python
maturin develop

# Or build wheels
maturin build --release
```

## Project Structure

```
bindings/python/
├── pyproject.toml          # Python package configuration with maturin
├── Cargo.toml              # Rust crate configuration
├── src/lib.rs              # PyO3 bindings code
├── python/wavemark/        # Python package structure
│   └── __init__.py         # Package initialization
├── examples/               # Usage examples
└── tests/                  # Python tests
```

## Key Features

- **Rust-powered**: Core functionality implemented in Rust for performance
- **PyO3 bindings**: Seamless Python integration
- **Maturin build system**: Automated wheel building for multiple platforms
- **Pre-compiled wheels**: No Rust toolchain required for end users
- **Cross-platform**: Supports Windows, macOS, and Linux

## Build Configuration

The `pyproject.toml` configures maturin as the build backend:

```toml
[build-system]
requires = ["maturin>=1.0,<2.0"]
build-backend = "maturin"

[tool.maturin]
python-source = "python"
module-name = "wavemark"
```

## Publishing to PyPI

1. **Build wheels for multiple platforms**:
   ```bash
   maturin build --release
   ```

2. **Upload to PyPI**:
   ```bash
   maturin publish
   ```

3. **Wheels are automatically built for**:
   - Windows (x86_64, i686)
   - macOS (x86_64, aarch64)
   - Linux (x86_64, i686, aarch64, armv7l)
   - Python 3.8, 3.9, 3.10, 3.11, 3.12

## Development

### Prerequisites
- Rust toolchain
- Python 3.8+
- maturin

### Local Development
```bash
# Install in development mode
maturin develop

# Run tests
python -m pytest tests/

# Run examples
python examples/basic_usage.py
```

### CI/CD Integration

The build process can be integrated into CI/CD pipelines to automatically build and publish wheels for new releases, similar to how tokenizers handles their releases.

## Usage

```python
import wavemark

# Basic usage
message = wavemark.hello_world()
print(message)
```

## Benefits of This Approach

1. **Performance**: Rust core provides high-performance audio processing
2. **Ease of use**: Simple `pip install wavemark` for end users
3. **No compilation**: Pre-built wheels eliminate build complexity
4. **Cross-platform**: Works on all major platforms
5. **Maintainable**: Clear separation between Rust core and Python bindings

This approach mirrors the successful pattern used by tokenizers, making wavemark easy to adopt in the Python ecosystem while maintaining the performance benefits of Rust.
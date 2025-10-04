# Wavemark TypeScript Bindings

TypeScript/WebAssembly bindings for the Wavemark audio watermarking library, optimized for web applications.

## Quick Start

```typescript
import { WavemarkEncoder, WavemarkDecoder } from './pkg/wavemark_typescript';

// Create encoder and decoder
const encoder = new WavemarkEncoder();
const decoder = new WavemarkDecoder();

// Generate sample audio
const audioData = new Float32Array(1000);
for (let i = 0; i < audioData.length; i++) {
    audioData[i] = Math.sin(2 * Math.PI * i / 1000);
}

// Encode watermark
const watermarked = encoder.encode(audioData, "AI_VOICE_v1.0");

// Decode watermark
const extracted = decoder.decode(watermarked);
console.log(`Extracted watermark: ${extracted}`);
```

## Installation

### For Web Applications
```bash
npm install
npm run build
```

### For Node.js Applications
```bash
npm install
npm run build:node
```

### For Bundlers (Webpack, Vite, etc.)
```bash
npm install
npm run build:bundler
```

## API Reference

### WavemarkEncoder
```typescript
const encoder = new WavemarkEncoder();
const watermarked = encoder.encode(audioData, watermark);
```

### WavemarkDecoder
```typescript
const decoder = new WavemarkDecoder();
const watermark = decoder.decode(audioData);
const isValid = decoder.verify(audioData, expectedWatermark);
```

### FourierProcessor
```typescript
const processor = new FourierProcessor();
const transformed = processor.transform(audioData);
```

### Utility Functions
```typescript
// Create sample audio data
const sampleAudio = create_sample_audio(44100, 1.0, 440); // 440Hz sine wave

// Standalone functions
const watermarked = encode_audio(audioData, watermark);
const watermark = decode_audio(audioData);
```

## Web Audio API Integration

```typescript
import { WebAudioWatermarker } from './examples/web_audio_integration';

const watermarker = new WebAudioWatermarker();

// Watermark microphone input
const stream = await watermarker.watermarkMicrophoneInput("LIVE_STREAM_v1.0");

// Watermark audio file
const watermarkedBuffer = await watermarker.watermarkAudioFile(file, "FILE_WATERMARK_v1.0");
```

## Examples

See the `examples/` directory for:
- Basic usage patterns
- Web Audio API integration
- Real-time audio processing
- File handling

## Development

```bash
# Run tests
npm test

# Build for different targets
npm run build        # Web
npm run build:node   # Node.js
npm run build:bundler # Bundlers

# Lint and format
npm run lint
npm run format
```

## Requirements

- Node.js 16+
- Rust toolchain (for building from source)
- wasm-pack (for WebAssembly compilation)

## Browser Support

- Chrome 57+
- Firefox 52+
- Safari 11+
- Edge 16+

## Performance

The WebAssembly bindings provide near-native performance:
- ~95% of native Rust performance
- Minimal JavaScript overhead
- Optimized for audio processing workloads

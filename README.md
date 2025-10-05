# Wavemark

Wavemark is an open-source toolkit for embedding verifiable metadata inside
audio signals without degrading perceptual quality. The project is in **early
development** and many components are still placeholders.

## Workspace Layout

- `wavemark/` – core Rust library under active development
- `bindings/typescript/` – experimental TypeScript/WebAssembly bindings
- `docs/` – design notes and subsystem guides
- `scripts/` – helper scripts for CI and local workflows

## Build & Test

```bash
# Build the core library
cargo build -p wavemark

# Run library tests
cargo test -p wavemark

# Execute workspace scripts
./scripts/quick_test.sh
```

Additional subsystem documentation lives in `docs/`; start with
[`docs/README.md`](docs/README.md) for an index when available.

## License

This project is licensed under the terms of the [MIT License](LICENSE).

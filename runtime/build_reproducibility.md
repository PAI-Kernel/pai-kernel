# Build Reproducibility (MAD-TR v1.0)

## Determinism Requirements
- `rust-toolchain.toml` pins compiler + components.
- `Cargo.lock` pins dependency graph.
- CI runs `cargo build --locked` and `cargo test --locked`.

## Local Repro Steps
```bash
rustc --version
cargo build --locked
cargo test --locked
```

If `Cargo.lock` changes unexpectedly, treat as a release-blocking event and require review.

# WeensyOS in Rust

SetUp Correct Target Architecture for a Rustc

```bash
rustup target add x86_64-unknown-none
```

```bash
rustup default nightly
```

```bash
cargo build --target x86_64-unknown-linux-gnu --release
```

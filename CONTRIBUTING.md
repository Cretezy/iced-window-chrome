# Contributing

## Setup

```bash
cargo fmt --all
cargo check --all-targets
```

## Before Opening a PR

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo check --all-targets
```

## Notes

- Windows and macOS behavior still depends on manual native validation.
- Linux behavior currently targets X11 Motif WM hints; Wayland remains no-op.
- Keep the crate surface small and runner-free unless there is a strong compatibility reason to expand it.

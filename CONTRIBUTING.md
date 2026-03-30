# Contributing

- Keep changes small and native to the platform they touch.

- Try to keep commits focused. One change per commit makes review and verification easier.

## Before opening a PR

```bash
cargo fmt --all --check
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
```

## Notes

- If you change Windows or macOS behavior, do a manual check on that OS.
- Linux support is X11-only right now. Wayland paths should stay harmless no-ops.
- Please avoid growing the crate into a custom runner unless there is a clear need and a real use case.

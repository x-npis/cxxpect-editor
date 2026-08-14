# Cxxpect Editor

A small native editor for `.cxxp` contracts. The first screen is the editor: file and edit menus, compact toolbar, line-number gutter, syntax/error highlighting, diagnostics, and status bar.

## Build

The project uses Rust stable and the public `cxxpect` API pinned to `v0.3.0`.

```text
cargo run
cargo test --all-targets
```

Release binaries for Windows, Linux, and macOS are produced by GitHub Actions. Windows users can run the downloaded `.exe` without installing Rust.

See [docs/architecture.md](docs/architecture.md) and [docs/user-guide.md](docs/user-guide.md).

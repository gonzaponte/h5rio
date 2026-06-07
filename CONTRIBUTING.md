# Contributing to h5rio

Thanks for considering a contribution. `h5rio` is intentionally small: changes
should keep the API focused on simple HDF5 table and fixed-shape array IO.

## Development Environment

The recommended environment is the Nix development shell. If you use `direnv`,
the repository `.envrc` can load it automatically. Otherwise, enter it
manually:

```bash
nix develop
```

The shell provides the pinned Rust toolchain, HDF5, `cargo-nextest`, `just`,
and the other tools used by CI.

Without Nix, install:

- the Rust toolchain listed in `rust-toolchain.toml`
- HDF5
- `cargo-nextest`, if you want to match the `just test` workflow exactly
- `just`, if you want to use the repository recipes

## Before Opening a Pull Request

With Nix and `just`, run:

```bash
just build
just test
just test-macros
just examples
```

Without Nix or `just`, run the equivalent commands:

```bash
cargo build
cargo test
cargo test -p h5rio_macros
cargo run --example array_round_trip
cargo run --example table_round_trip
```

## Code Guidelines

- Prefer small, explicit APIs over broad abstractions.
- Keep HDF5-owned types prefixed as `hdf5::...` in public signatures.
- Public library code should return `hdf5::Result` instead of panicking.
- Validate user-provided shapes before passing them to HDF5 when practical.
- Add focused tests for new behavior and regressions.
- Keep README examples and Rustdoc in sync with public API changes.

## Versioning and Releases

The project follows semantic versioning. Public API changes should be reflected
in `CHANGELOG.md` and in both crate versions when preparing a release:

- `Cargo.toml`
- `macros/Cargo.toml`
- the root dependency on `h5rio_macros`
- `Cargo.lock`

Release tags use the form `vX.Y.Z`.

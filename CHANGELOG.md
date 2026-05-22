# Changelog

All notable changes to this project are tracked here.

## Unreleased

### Added

- CI workflow for formatting, clippy, tests, docs, security checks, and release build smoke coverage.
- Security CI job running `cargo audit` through RustSec audit-check and `cargo deny check` through cargo-deny.
- Release build smoke test with `cargo build --workspace --release`.
- Shell completion generation through `xmind completion <shell>`.
- Added installation documentation for source installs, release binary builds, shell completion setup, and local verification.

### Changed

- Command, schema, technical, and example documentation now track the implemented CLI surface.
- `src/app/mod.rs` has begun moving command-specific rendering into focused app modules.

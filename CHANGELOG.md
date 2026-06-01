# Changelog

All notable changes to this project are tracked here.

## Unreleased

## v0.1.1 - 2026-06-01

### Fixed

- Markdown export now writes topic notes as body text below each topic heading.
- Markdown export now preserves topic hyperlinks as heading links with angle-bracket destinations.

## v0.1.0 - 2026-05-28

### Added

- CI workflow for formatting, clippy, tests, docs, security checks, and release build smoke coverage.
- Security CI job running `cargo audit` through RustSec audit-check and `cargo deny check` through cargo-deny.
- Release build smoke test with `cargo build --workspace --release`.
- Minimal cargo-dist release workflow for tag-driven GitHub Release archives and SHA256 checksum artifacts.
- Shell completion generation through `xmind completion <shell>`.
- Added installation documentation for source installs, release binary builds, shell completion setup, and local verification.
- Added `scripts/install.sh` with dry-run preview, supported platform artifact selection, SHA256SUMS verification, and actionable failure messages.
- Added release policy documentation for version tags, changelog ownership, GitHub Release notes, and SHA256 checksum verification.
- Added Homebrew tap publication for tagged releases through `ivan-94/homebrew-tap`.
- Restore command support for dry-run and apply from the newest matching `.xmind-backups` entry.

### Changed

- Command, schema, technical, and example documentation now track the implemented CLI surface.
- `src/app/mod.rs` has begun moving command-specific rendering into focused app modules.

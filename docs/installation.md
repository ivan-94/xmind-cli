# Installation

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Local and release installation instructions
- GitHub issue #7: `https://github.com/ivan-94/xmind-cli/issues/7`
- Release policy: `technical/release-policy.md`
- Last updated: 2026-05-23

## Prerequisites

- Rust stable toolchain.
- A local checkout of this repository.

## Install From Source

From the repository root:

```bash
cargo install --path .
```

This installs the `xmind` binary into Cargo's configured binary directory, usually
`~/.cargo/bin`.

Verify the installed binary:

```bash
xmind --version
```

## Build A Release Binary

For a local release artifact without installing it:

```bash
cargo build --workspace --release
```

The binary is written to:

```bash
target/release/xmind
```

Verify that artifact directly:

```bash
target/release/xmind --version
```

## Verify GitHub Release Downloads

When GitHub Release artifacts are available, download both the artifact and
the `SHA256SUMS` file from the same release page.

From the download directory:

```bash
shasum -a 256 -c SHA256SUMS
```

Run the binary only after the downloaded artifact reports `OK`.

The release policy in `technical/release-policy.md` defines version tags,
changelog source of truth, release note updates, and checksum publication
rules. The first release does not publish to crates.io, and install script or
Homebrew instructions should appear here only after their release slices land.

## Shell Completion

Generate a completion script and install it using your shell's standard
completion location. For example:

```bash
xmind completion bash > xmind.bash
xmind completion zsh > _xmind
```

Supported shell values are documented in `reference/commands/completion.md`.

## Local Quality Check

Before sharing a build, run:

```bash
./scripts/quality-gate.sh
```

CI also runs a release build smoke test with:

```bash
cargo build --workspace --release
```

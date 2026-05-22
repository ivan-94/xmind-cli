# Installation

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Local and release installation instructions
- Last updated: 2026-05-22

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

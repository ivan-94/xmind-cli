# Installation

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Local and release installation instructions
- GitHub issue #7: `https://github.com/ivan-94/xmind-cli/issues/7`
- GitHub issue #5: `https://github.com/ivan-94/xmind-cli/issues/5`
- GitHub issue #6: `https://github.com/ivan-94/xmind-cli/issues/6`
- GitHub issue #9: `https://github.com/ivan-94/xmind-cli/issues/9`
- Release policy: `technical/release-policy.md`
- Cargo-dist config: `../Cargo.toml` `[workspace.metadata.dist]`
- Cargo-dist workflow: `../.github/workflows/release.yml`
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

When GitHub Release artifacts are available, choose the artifact for one of the
supported release platforms:

| Platform | Target |
| --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` |
| macOS Intel | `x86_64-apple-darwin` |
| Linux x86_64 GNU | `x86_64-unknown-linux-gnu` |
| Windows x86_64 MSVC | `x86_64-pc-windows-msvc` |

Download both the artifact and the `SHA256SUMS` file from the same release page.

From the download directory:

```bash
shasum -a 256 -c SHA256SUMS
```

Run the binary only after the downloaded artifact reports `OK`.

Smoke-test the downloaded binary before using it on real workbooks:

```bash
xmind --version
xmind tree tests/fixtures/xmind/minimal.xmind --json
xmind validate tests/fixtures/xmind/minimal.xmind --json
```

The first GitHub Release does not publish Linux arm64, Linux musl/static builds,
macOS universal binaries, 32-bit Windows, Windows GNU, crates.io packages,
container images, Homebrew formulae, or install script artifacts. Homebrew and
install script instructions will be added only after their separate release
slices land.

The release policy in `technical/release-policy.md` defines version tags,
changelog source of truth, release note updates, and checksum publication
rules. The cargo-dist workflow currently publishes release archives with
per-artifact `.sha256` checksum files from `v*` tags. The first release does
not publish to crates.io, and install script or Homebrew instructions should
appear here only after their release slices land.

## Future Homebrew Channel

Homebrew is a future install channel, not an available installation path today.
The planned tap repository is `ivan-94/homebrew-tap`.

Enable Homebrew instructions only after all of these are true:

1. A GitHub Release artifact exists for the formula URL.
2. The formula checksum matches the published GitHub Release checksum for that artifact.
3. The formula installs the `xmind` binary and its test runs `xmind --version`.
4. The tap formula passes:

```bash
brew audit --strict --online
brew test
```

Until those checks pass, install from source or use a verified GitHub Release
download when one is available.

## Verify Release Automation Locally

Install the pinned cargo-dist version if needed:

```bash
cargo install cargo-dist --version 0.31.0 --locked
```

Check the tag-driven GitHub Release plan without publishing:

```bash
cargo dist plan
```

Build local release artifacts without creating a GitHub Release:

```bash
cargo dist build --artifacts=local --target aarch64-apple-darwin
cargo dist build --artifacts=local --target x86_64-apple-darwin
cargo dist build --artifacts=local --target x86_64-unknown-linux-gnu
cargo dist build --artifacts=local --target x86_64-pc-windows-msvc
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

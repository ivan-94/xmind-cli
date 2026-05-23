# Installation

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Source, GitHub Release binary, install script, and local release installation instructions
- GitHub issue #7: `https://github.com/ivan-94/xmind-cli/issues/7`
- GitHub issue #5: `https://github.com/ivan-94/xmind-cli/issues/5`
- GitHub issue #6: `https://github.com/ivan-94/xmind-cli/issues/6`
- GitHub issue #8: `https://github.com/ivan-94/xmind-cli/issues/8`
- Release policy: `technical/release-policy.md`
- Cargo-dist config: `../Cargo.toml` `[workspace.metadata.dist]`
- Cargo-dist workflow: `../.github/workflows/release.yml`
- Install script: `../scripts/install.sh`
- Last updated: 2026-05-23

## Choose An Install Channel

- Use Cargo source install when you already have Rust and want the simplest path from source.
- Use GitHub Release binaries when you want a prebuilt artifact for a supported platform.
- Use the install script when you want the script to pick the current platform artifact, verify checksums, and copy the binary into your user bin directory.
- Use a local release build when developing or validating release behavior from a checkout.

## Prerequisites

For Cargo source installs:

- Rust stable toolchain.

For GitHub Release binary downloads:

- A supported release platform: macOS Apple Silicon, macOS Intel, Linux x86_64 GNU, or Windows x86_64 MSVC.
- `shasum` for checksum verification on macOS/Linux. Windows users can use `certutil -hashfile <artifact> SHA256` and compare against `SHA256SUMS`.

For the install script:

- Bash, `curl`, `shasum`, and `tar` on macOS/Linux.
- A tagged GitHub Release that publishes the matching archive and `SHA256SUMS`.

## Install From Source

Install directly from GitHub:

```bash
cargo install --locked --git https://github.com/ivan-94/xmind-cli
```

From a local repository checkout:

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

## Install From GitHub Release Binary

When GitHub Release artifacts are available, choose the artifact for one of the
supported release platforms:

| Platform | Target |
| --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` |
| macOS Intel | `x86_64-apple-darwin` |
| Linux x86_64 GNU | `x86_64-unknown-linux-gnu` |
| Windows x86_64 MSVC | `x86_64-pc-windows-msvc` |

The release archive names are expected to follow this pattern:

```text
xmind-cli-vX.Y.Z-<target>.tar.gz
xmind-cli-vX.Y.Z-x86_64-pc-windows-msvc.zip
```

Download both the artifact and the `SHA256SUMS` file from the same release page.

From the download directory on macOS/Linux:

```bash
shasum -a 256 -c SHA256SUMS
```

Run the binary only after the downloaded artifact reports `OK`. Extract the archive, move `xmind` or `xmind.exe` into a directory on your `PATH`, and then verify it.

Smoke-test the downloaded binary before using it on real workbooks:

```bash
xmind --version
xmind tree tests/fixtures/xmind/minimal.xmind --json
xmind validate tests/fixtures/xmind/minimal.xmind --json
```

## Install With The Script

Preview the selected platform artifact and install path without writing files:

```bash
bash scripts/install.sh --dry-run --version v0.1.0
```

Install into `~/.local/bin` after checksum verification:

```bash
bash scripts/install.sh --version v0.1.0
```

Install into a custom directory:

```bash
bash scripts/install.sh --version v0.1.0 --install-dir "$HOME/bin"
```

The script maps the current OS and CPU to the supported release target, downloads `SHA256SUMS`, verifies the exact artifact entry, extracts the archive, and copies the binary to the install directory. It refuses unsupported platforms, missing checksum entries, checksum mismatches, and archives that do not contain the expected binary. Dry-run mode does not download, extract, or write user files.

The first GitHub Release does not publish Linux arm64, Linux musl/static builds,
macOS universal binaries, 32-bit Windows, Windows GNU, crates.io packages,
container images, or Homebrew formulae. Homebrew instructions will be added only
after its separate release slice lands.

The release policy in `technical/release-policy.md` defines version tags,
changelog source of truth, release note updates, and checksum publication
rules. The cargo-dist workflow currently publishes release archives with
per-artifact `.sha256` checksum files from `v*` tags, while this script consumes
the aggregate `SHA256SUMS` convention documented by the release policy. The first
release does not publish to crates.io, and Homebrew instructions should appear
here only after its release slice lands.

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

# Installation

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Source, GitHub Release binary, install script, Homebrew, and local release installation instructions
- GitHub issue #7: `https://github.com/ivan-94/xmind-cli/issues/7`
- GitHub issue #5: `https://github.com/ivan-94/xmind-cli/issues/5`
- GitHub issue #6: `https://github.com/ivan-94/xmind-cli/issues/6`
- GitHub issue #8: `https://github.com/ivan-94/xmind-cli/issues/8`
- GitHub issue #9: `https://github.com/ivan-94/xmind-cli/issues/9`
- Release policy: `technical/release-policy.md`
- Cargo-dist config: `../Cargo.toml` `[workspace.metadata.dist]`
- Cargo-dist workflow: `../.github/workflows/release.yml`
- Install script: `../scripts/install.sh`
- Parent review fix on 2026-05-23: release workflow must publish the aggregate `SHA256SUMS` consumed by the install script.
- Last updated: 2026-05-23

## Choose An Install Channel

- Use Cargo source install when you already have Rust and want the simplest path from source.
- Use GitHub Release binaries when you want a prebuilt artifact for a supported platform.
- Use the install script when you want the script to pick the current platform artifact, verify checksums, and copy the binary into your user bin directory.
- Use Homebrew when you want package-manager updates on macOS or Linuxbrew.
- Use a local release build when developing or validating release behavior from a checkout.

## Prerequisites

For Cargo source installs:

- Rust stable toolchain.

For GitHub Release binary downloads:

- A supported release platform: macOS Apple Silicon, macOS Intel, Linux x86_64 GNU, Linux arm64 GNU, or Windows x86_64 MSVC.
- `shasum` for checksum verification on macOS/Linux. Windows users can use `certutil -hashfile <artifact> SHA256` and compare against `SHA256SUMS`.

For the install script:

- Bash, `curl`, `shasum`, and `tar` on macOS/Linux.
- A tagged GitHub Release from this workflow; it publishes the matching archive and aggregate `SHA256SUMS` together.

For Homebrew:

- Homebrew on macOS or Linuxbrew.
- A tagged GitHub Release whose Homebrew formula has been published to `ivan-94/homebrew-tap`.

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
| Linux arm64 GNU | `aarch64-unknown-linux-gnu` |
| Windows x86_64 MSVC | `x86_64-pc-windows-msvc` |

The release archive names are expected to follow this pattern:

```text
xmind-cli-<target>.tar.gz
xmind-cli-x86_64-pc-windows-msvc.zip
```

Download both the artifact and the aggregate `SHA256SUMS` file from the same release page. The checked-in release workflow generates `SHA256SUMS` from the actual `target/distrib` release archives before uploading `target/distrib/*`.

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

## Install With Homebrew

Install the latest tagged release from the project tap:

```bash
brew install ivan-94/tap/xmind-cli
```

Verify the installed binary:

```bash
xmind --version
```

The Homebrew formula installs the `xmind` executable from the published
GitHub Release archive for your OS and CPU. The release workflow updates
`ivan-94/homebrew-tap` after the GitHub Release artifacts and `SHA256SUMS` file
exist, so the formula URL and SHA256 values come from the same release.

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

The first GitHub Release does not publish Linux musl/static builds, macOS
universal binaries, 32-bit Windows, Windows GNU, crates.io packages, or
container images.

The release policy in `technical/release-policy.md` defines version tags,
changelog source of truth, release note updates, and checksum publication
rules. The cargo-dist workflow publishes release archives with per-artifact `.sha256` checksum files from `v*` tags, then the GitHub Release job generates and uploads aggregate `SHA256SUMS` for script, manual, and Homebrew verification. The first
release does not publish to crates.io.

## Homebrew Publisher Maintenance

The Homebrew tap repository is `ivan-94/homebrew-tap`. Homebrew is an active
Homebrew channel for tagged releases once the release workflow has a repository
secret named `HOMEBREW_TAP_TOKEN` with write access to that tap.

The release workflow runs `.github/scripts/update-homebrew-formula.sh` after the
GitHub Release job. The script reads `target/distrib/SHA256SUMS`, writes
`Formula/xmind-cli.rb`, and pushes it to the tap. The generated formula must:

1. Use the matching GitHub Release artifact URLs.
2. Copy SHA256 values from the same release's `SHA256SUMS` file.
3. Install the `xmind` binary.
4. Test `xmind --version`.
5. Pass tap validation:

```bash
brew audit --strict --online
brew test ivan-94/tap/xmind-cli
```

## Verify Release Automation Locally

Install the pinned cargo-dist version if needed:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/cargo-dist/releases/download/v0.31.0/cargo-dist-installer.sh | sh
```

Check the tag-driven GitHub Release plan without publishing:

```bash
cargo dist plan
```

Build local release artifacts without creating a GitHub Release:

```bash
cargo dist build --artifacts=local --target aarch64-apple-darwin
cargo dist build --artifacts=local --target aarch64-unknown-linux-gnu
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

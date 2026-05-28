# Release Policy

## Source Manifest

### Sources

- GitHub issue #7: `https://github.com/ivan-94/xmind-cli/issues/7`
- GitHub issue #5: `https://github.com/ivan-94/xmind-cli/issues/5`
- GitHub issue #6: `https://github.com/ivan-94/xmind-cli/issues/6`
- GitHub issue #9: `https://github.com/ivan-94/xmind-cli/issues/9`
- Parent PRD #1: `https://github.com/ivan-94/xmind-cli/issues/1`
- `PLAN.md` Phase 18 release automation and Homebrew sections.
- `Cargo.toml` `[workspace.metadata.dist]`
- `.github/workflows/release.yml`
- Parent review instruction on 2026-05-23 for issue #8: fix the end-to-end checksum contract so the release workflow publishes the checksum source consumed by `scripts/install.sh`.
- `CHANGELOG.md`
- `docs/installation.md`
- `docs/technical/e2e-test-plan.md`
- `docs/technical/quality-gates.md`
- `docs/prd/1/implementation-notes.html`
- User slice instruction on 2026-05-23: document policy only; do not implement cargo-dist, install script, Homebrew, or platform matrix.
- User slice instruction on 2026-05-23 for issue #5: configure cargo-dist minimally for GitHub Releases from tags, generated checksums, changelog/release notes source, and no Homebrew requirement.
- User slice instruction on 2026-05-23 for issue #6: define the release platform matrix, add binary smoke checks, keep full E2E separate, and do not implement install script or Homebrew.
- User slice instruction on 2026-05-23 for issue #8: add a pragmatic install script, document Cargo Git source install, GitHub Release binary download, script install, dry-run preview, platform artifact selection, checksum verification, and actionable errors.
- User slice instruction on 2026-05-23 for issue #9: choose a conservative Homebrew path, document tap ownership and enablement conditions, and do not claim Homebrew is currently available without a verifiable release artifact.

### Produced Artifacts

- `docs/technical/release-policy.md`
- `docs/installation.md`
- `docs/README.md`
- `docs/technical/README.md`
- `CHANGELOG.md`
- `docs/prd/1/implementation-notes.html`
- `Cargo.toml`
- `.github/workflows/release.yml`
- `tests/cli/doc_examples_test.rs`
- `tests/cli/release_workflow_test.rs`
- `scripts/install.sh`
- `tests/cli/install_script_test.rs`

### Key Decisions

- GitHub Releases are the first public release channel.
- `CHANGELOG.md` is the source of truth for user-facing release notes.
- The first release uses `v0.1.0`; later tags use `vMAJOR.MINOR.PATCH`.
- Release artifacts publish one `SHA256SUMS` file containing SHA-256 digests for every downloadable binary archive and installer artifact.
- crates.io publishing is out of scope for the first release.
- issue #5 uses `cargo-dist` `0.31.0` with `ci = ["github"]`, `hosting = ["github"]`, `create-release = true`, and `installers = []`.
- issue #6 defines the first release binary matrix as macOS Apple Silicon, macOS Intel, Linux x86_64 GNU, and Windows x86_64 MSVC.
- release jobs smoke-test each produced native binary with `xmind --version`, `xmind tree ... --json`, and `xmind validate ... --json`.
- Full E2E matrix remains separate from release binary smoke checks.
- cargo-dist emits per-artifact `.sha256` checksum files. The `github-release` job generates `SHA256SUMS` from the actual downloaded `target/distrib` release archives before publication, and the standalone install script consumes that aggregate checksum file.
- issue #8 adds `scripts/install.sh` as a repository-maintained installer, separate from cargo-dist generated installers; `[workspace.metadata.dist].installers` remains empty.
- `ivan-94/homebrew-tap` is the Homebrew tap repository. The checked-in release workflow publishes `Formula/xmind-cli.rb` after GitHub Release artifacts and `SHA256SUMS` exist.

### Verification Evidence

- Documentation policy is guarded by `tests/cli/doc_examples_test.rs`.
- Release workflow configuration, including aggregate `SHA256SUMS` generation before release publication, is guarded by `tests/cli/release_workflow_test.rs`.
- Install script behavior is guarded by `tests/cli/install_script_test.rs`.
- RED: `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test doc_examples_test release_policy_documents_versioning_changelog_notes_and_checksums` failed before `docs/technical/release-policy.md` existed.
- GREEN: `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test doc_examples_test release_policy_documents_versioning_changelog_notes_and_checksums` passed after adding the policy and checksum docs.
- Full targeted docs check: `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test doc_examples_test`.
- Local quality gate: `PATH=/opt/homebrew/opt/rustup/bin:$PATH ./scripts/quality-gate.sh`.

### Open Questions / Risks

- The install script and Homebrew formula expect cargo-dist release archive names to follow `xmind-cli-<target>.tar.gz` for macOS/Linux and `xmind-cli-x86_64-pc-windows-msvc.zip` for Windows.
- cargo-dist local verification requires installing `cargo-dist` 0.31.0 locally when it is not already present.
- Homebrew formula checksum updates depend on published release artifact names and the `HOMEBREW_TAP_TOKEN` secret.
- Homebrew tap publication still depends on the separate `ivan-94/homebrew-tap` repository existing before the first tagged release.

## Versioning

Tags use SemVer with a leading `v`, starting at `v0.1.0`.

Before every release:

1. Choose the next version.
2. Set `Cargo.toml` `package.version` to the tag value without the leading `v`.
3. Create a Git tag named `vMAJOR.MINOR.PATCH`.
4. Confirm `xmind --version` reports the same version after the release binary is built.

Pre-1.0 releases may include breaking command, schema, or output changes in a minor version bump. Patch releases are reserved for bug fixes, documentation corrections, and packaging fixes that do not intentionally change public CLI behavior.

After `v1.0.0`, breaking changes require a major version bump. Minor versions add compatible features, and patch versions fix bugs or release packaging without changing the documented contract.

## Changelog

`CHANGELOG.md` is the source of truth for user-facing release notes.

During development, changes land under `## Unreleased`. Before tagging:

1. Move the relevant `Unreleased` entries into `## vX.Y.Z - YYYY-MM-DD`.
2. Keep empty or future work under a new `## Unreleased` section.
3. Group entries by user-visible impact: added, changed, fixed, removed, security, or documentation.
4. Do not copy internal implementation details unless they explain a user-visible behavior or migration.

The GitHub Release notes should be generated from the matching `CHANGELOG.md` section, then edited only to add download links, checksum instructions, and known installation caveats.

## Release Notes Update Process

For each GitHub Release:

1. Verify `Cargo.toml` matches the tag without `v`.
2. Verify `CHANGELOG.md` has a `## vX.Y.Z - YYYY-MM-DD` section.
3. Build and smoke-test release artifacts according to the cargo-dist release workflow.
4. Publish binary archives and the `SHA256SUMS` checksum file.
5. Paste the matching changelog section into the GitHub Release description.
6. Add a short install section that links to `docs/installation.md`.
7. Add checksum verification commands using `SHA256SUMS`.

Release notes must not claim crates.io or platform support before those channels are implemented and validated. The cargo-dist GitHub Release body is generated from the matching `CHANGELOG.md` version section for the pushed tag; release publication fails if that section is missing.

## Checksums

Every GitHub Release must publish checksums next to the downloadable artifacts. cargo-dist generates per-artifact `.sha256` checksum files, and the checked-in `github-release` job also generates an aggregate `SHA256SUMS` file from the actual release archives present in `target/distrib` before publishing. The aggregate file contains one line per artifact:

```text
<sha256>  <artifact-file-name>
```

The release workflow creates `SHA256SUMS` in `target/distrib` after downloading all matrix artifacts and before the `softprops/action-gh-release` upload. The upload step publishes `target/distrib/*`, so the aggregate checksum file and the archives are attached to the same GitHub Release.

Users verify downloads from the directory containing the downloaded artifact and `SHA256SUMS`:

```bash
shasum -a 256 -c SHA256SUMS
```

The command should report `OK` for the downloaded artifact. If checksum verification fails, delete the artifact, download it again from the GitHub Release, and do not run the binary until verification passes.

Homebrew formula checksums must come from the same published GitHub Release artifact checksums. Do not invent separate checksum values for the tap.

## cargo-dist Workflow

The first checked-in cargo-dist configuration lives in `Cargo.toml` under `[workspace.metadata.dist]`.

The issue #5 configuration starts the GitHub Release workflow. Issue #6 expands it to the first supported binary platform matrix:

- `cargo-dist-version = "0.31.0"` pins the release automation generator.
- `allow-dirty = ["ci"]` is intentional because this repository keeps a
  hand-maintained release workflow with custom release notes extraction,
  aggregate `SHA256SUMS`, artifact smoke tests, and the repository install
  script policy instead of accepting cargo-dist's generated workflow verbatim.
- `ci = ["github"]` and `hosting = ["github"]` make GitHub Actions and GitHub Releases the release path.
- `create-release = true` keeps releases tag-driven from `v*` tags.
- `checksum = "sha256"` publishes per-artifact `.sha256` checksum files.
- CI installs the pinned cargo-dist binary with the official `v0.31.0`
  `cargo-dist-installer.sh` release installer and adds the install directory to
  `GITHUB_PATH`; the `axodotdev/cargo-dist` repository tag is not used as a
  GitHub Action.
- The `github-release` job additionally generates aggregate `SHA256SUMS` from `target/distrib` archives before publishing `target/distrib/*`.
- `installers = []` avoids cargo-dist generated installers; the standalone `scripts/install.sh` and `.github/scripts/update-homebrew-formula.sh` remain repository-maintained.
- `targets` contains `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, and `x86_64-pc-windows-msvc`.

## Supported Release Platforms

The first GitHub Release binary matrix is:

| Platform | cargo-dist target | Runner | Binary smoke |
| --- | --- | --- | --- |
| macOS Apple Silicon | `aarch64-apple-darwin` | `macos-14` | `xmind --version`, `xmind tree tests/fixtures/xmind/minimal.xmind --json`, `xmind validate tests/fixtures/xmind/minimal.xmind --json` |
| macOS Intel | `x86_64-apple-darwin` | `macos-15-intel` | Same smoke commands |
| Linux x86_64 GNU | `x86_64-unknown-linux-gnu` | `ubuntu-latest` | Same smoke commands |
| Linux arm64 GNU | `aarch64-unknown-linux-gnu` | `ubuntu-24.04-arm` | Same smoke commands |
| Windows x86_64 MSVC | `x86_64-pc-windows-msvc` | `windows-latest` | Same smoke commands |

The smoke suite proves the produced binary starts, reports its version, reads a minimal workbook tree as JSON, and validates the same workbook as JSON. It is intentionally not the full E2E matrix; command-by-command coverage remains in the Rust E2E suite and its release/nightly expansion.

## Unsupported Platforms

Do not imply downloadable release support for platforms outside the table above. In particular, the first release does not promise Linux musl/static builds, macOS universal binaries, 32-bit Windows, Windows GNU, crates.io, or container images.

Users on unsupported platforms may build from source with Rust when their target is compatible with the codebase, but those builds are not release artifacts until a later platform slice adds them to the supported matrix and smoke checks.

Local verification commands:

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/axodotdev/cargo-dist/releases/download/v0.31.0/cargo-dist-installer.sh | sh
cargo dist plan
cargo dist build --artifacts=local --target aarch64-apple-darwin
cargo dist build --artifacts=local --target aarch64-unknown-linux-gnu
cargo dist build --artifacts=local --target x86_64-apple-darwin
cargo dist build --artifacts=local --target x86_64-unknown-linux-gnu
cargo dist build --artifacts=local --target x86_64-pc-windows-msvc
xmind --version
xmind tree tests/fixtures/xmind/minimal.xmind --json
xmind validate tests/fixtures/xmind/minimal.xmind --json
```

## Install Script Policy

`scripts/install.sh` is the supported script entrypoint for GitHub Release archives. It must:

- map only the supported release platforms to artifacts;
- preview release tag, target, artifact URL, checksum URL, and install path in `--dry-run` without writing user files;
- rely on the `SHA256SUMS` file published by the checked-in GitHub Release workflow;
- verify `SHA256SUMS` before extraction;
- fail when the checksum file does not contain the exact artifact name;
- fail on checksum mismatch before installing anything;
- fail when an archive does not contain the expected `xmind` or `xmind.exe` binary.

The script is not a cargo-dist generated installer and does not change the crates.io non-goal.

## Homebrew Tap

The Homebrew tap repository is `ivan-94/homebrew-tap`. The repository
name follows Homebrew tap convention while keeping ownership under the same
GitHub account as `ivan-94/xmind-cli`.

Homebrew is an active Homebrew channel for tagged releases. Users install the
formula with:

```bash
brew install ivan-94/tap/xmind-cli
```

The repository-maintained release workflow publishes the formula after the
GitHub Release job completes. It does not use cargo-dist generated Homebrew
publish jobs because this repository already has custom release-note extraction,
aggregate `SHA256SUMS`, release smoke checks, and install-script policy.

Before the first tagged release, create `ivan-94/homebrew-tap` and add a
`HOMEBREW_TAP_TOKEN` repository secret to `ivan-94/xmind-cli`. The token must
have write access to the tap repository.

The Homebrew formula must satisfy all of these conditions:

1. A versioned GitHub Release artifact exists for the macOS formula URL.
2. The formula `sha256` is copied from the same published GitHub Release
   artifact checksum. Homebrew formula checksums must come from the same
   published GitHub Release artifact checksums; do not invent or manually
   diverge checksum values.
3. The formula installs the `xmind` binary.
4. The formula test runs `xmind --version`.
5. The formula passes:

```bash
brew audit --strict --online
brew test ivan-94/tap/xmind-cli
```

The formula update path sets the version, URL, and checksum from the published
release metadata in one change. If those inputs are unavailable, the Homebrew
publish job must fail instead of publishing a placeholder formula.

## First Release Non-Goals

- Do not publish to crates.io for the first release.
- Do not add release artifact support beyond the issue #6 platform matrix without matching smoke checks and documentation.

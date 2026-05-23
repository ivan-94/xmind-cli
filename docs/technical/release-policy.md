# Release Policy

## Source Manifest

### Sources

- GitHub issue #7: `https://github.com/ivan-94/xmind-cli/issues/7`
- Parent PRD #1: `https://github.com/ivan-94/xmind-cli/issues/1`
- `PLAN.md` Phase 18 release automation and Homebrew sections.
- `CHANGELOG.md`
- `docs/installation.md`
- `docs/technical/e2e-test-plan.md`
- `docs/technical/quality-gates.md`
- `docs/prd/1/implementation-notes.html`
- User slice instruction on 2026-05-23: document policy only; do not implement cargo-dist, install script, Homebrew, or platform matrix.

### Produced Artifacts

- `docs/technical/release-policy.md`
- `docs/installation.md`
- `docs/README.md`
- `docs/technical/README.md`
- `CHANGELOG.md`
- `docs/prd/1/implementation-notes.html`
- `tests/cli/doc_examples_test.rs`

### Key Decisions

- GitHub Releases are the first public release channel.
- `CHANGELOG.md` is the source of truth for user-facing release notes.
- The first release uses `v0.1.0`; later tags use `vMAJOR.MINOR.PATCH`.
- Release artifacts publish one `SHA256SUMS` file containing SHA-256 digests for every downloadable binary archive and installer artifact.
- crates.io publishing is out of scope for the first release.

### Verification Evidence

- Documentation policy is guarded by `tests/cli/doc_examples_test.rs`.
- RED: `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test doc_examples_test release_policy_documents_versioning_changelog_notes_and_checksums` failed before `docs/technical/release-policy.md` existed.
- GREEN: `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test doc_examples_test release_policy_documents_versioning_changelog_notes_and_checksums` passed after adding the policy and checksum docs.
- Full targeted docs check: `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test doc_examples_test`.
- Local quality gate: `PATH=/opt/homebrew/opt/rustup/bin:$PATH ./scripts/quality-gate.sh`.

### Open Questions / Risks

- Final artifact names depend on the cargo-dist slice and platform matrix slice.
- Homebrew formula checksum updates depend on published release artifact names.

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
3. Build and smoke-test release artifacts according to the release workflow.
4. Publish binary archives and the `SHA256SUMS` checksum file.
5. Paste the matching changelog section into the GitHub Release description.
6. Add a short install section that links to `docs/installation.md`.
7. Add checksum verification commands using `SHA256SUMS`.

Release notes must not claim crates.io, Homebrew, install script, or platform support before those channels are implemented and validated.

## Checksums

Every GitHub Release must publish a `SHA256SUMS` file next to the downloadable artifacts. The file contains one line per artifact:

```text
<sha256>  <artifact-file-name>
```

Generate or verify this file through the release automation once cargo-dist is configured. Until then, manual release candidates may use platform tools such as:

```bash
shasum -a 256 xmind-cli-*.tar.gz xmind-cli-*.zip > SHA256SUMS
```

Users verify downloads from the directory containing the downloaded artifact and `SHA256SUMS`:

```bash
shasum -a 256 -c SHA256SUMS
```

The command should report `OK` for the downloaded artifact. If checksum verification fails, delete the artifact, download it again from the GitHub Release, and do not run the binary until verification passes.

Homebrew formula checksums must come from the same published GitHub Release artifact checksums. Do not invent separate checksum values for the tap.

## First Release Non-Goals

- Do not publish to crates.io for the first release.
- Do not promise Homebrew availability before the tap formula is merged.
- Do not document an install script as available before issue #8 implements and verifies it.
- Do not freeze the final platform matrix in this policy; use the platform matrix slice as the source of truth once it lands.

# xmind-cli

[中文](README.zh-CN.md)

`xmind-cli` is an unofficial command line interface for inspecting, querying,
editing, validating, and exporting XMind workbooks. The repository name is
`xmind-cli`; the installed executable is `xmind`.

The project is built for human and AI-assisted workflows where a mind map needs
to behave like structured data instead of an opaque zip package. Agents can
inspect a workbook, locate topics with stable selectors, preview mutations with
JSON diffs, apply changes explicitly, and validate the result before handing work
back to a human.

This project is not affiliated with, endorsed by, or sponsored by XMind.

## Status

`xmind-cli` is an early-release project. The package version is `0.1.0`,
`Cargo.toml` sets `publish = false`, and the current changelog is still under
`Unreleased`. The CLI is usable from source and has integration coverage for the
implemented command surface, but public release artifacts and package-manager
distribution are still evolving.

Use it with real workbooks only after running a dry run, checking the JSON or
human-readable diff, and keeping a backup of the original file.

## Capabilities

- Inspect workbook structure, sheets, topics, metadata, and validation state.
- Find and get topics by title, id, or path-style selectors.
- Preview safe mutations with `--dry-run` and machine-readable `--json` output.
- Apply supported topic and subtree edits with explicit `--apply`.
- Create backups and restore from matching `.xmind-backups` entries.
- Export supported workbook content to formats documented in the command
  reference.
- Generate shell completions with `xmind completion <shell>`.

## Install Or Build

Install from the current GitHub source:

```bash
cargo install --locked --git https://github.com/ivan-94/xmind-cli
```

Build from a local checkout:

```bash
cargo build --workspace --release
target/release/xmind --version
```

During development, run from the checkout:

```bash
cargo run -- tree tests/fixtures/xmind/minimal.xmind --depth 2
```

Tagged GitHub Release archives and the install script are part of the release
flow, but the project has not yet settled into a stable package-manager channel.
See [docs/installation.md](docs/installation.md) for current install, release
build, checksum, and shell completion details.

| Channel | Status | Notes |
| --- | --- | --- |
| Cargo source install from checkout | Available today | `cargo install --path .` |
| Cargo source install from GitHub | Available today | `cargo install --locked --git https://github.com/ivan-94/xmind-cli` |
| Local release build | Available today | `cargo build --workspace --release`, then run `target/release/xmind` |
| GitHub Release binaries | Available after the first tagged release | Planned targets: macOS Apple Silicon, macOS Intel, Linux x86_64 GNU, Linux arm64 GNU, and Windows x86_64 MSVC. |
| Install script | Available for tagged release artifacts | `bash scripts/install.sh --dry-run --version v0.1.0`, then rerun without `--dry-run`. |
| Homebrew tap | Planned | Expected path is `ivan-94/homebrew-tap`. Formula publication waits for a verified GitHub Release artifact and checksum. |

The first binary release matrix does not imply support for: Linux musl/static builds, macOS universal binaries, 32-bit Windows, Windows GNU, container images, Homebrew, or crates.io packages.

## Quick Start

Inspect a committed fixture:

```bash
xmind inspect tests/fixtures/xmind/minimal.xmind --json
xmind sheets tests/fixtures/xmind/minimal.xmind --json
xmind tree tests/fixtures/xmind/minimal.xmind --depth 2 --json
```

Find and read topics:

```bash
xmind find tests/fixtures/xmind/minimal.xmind --title "Payment" --json
xmind get tests/fixtures/xmind/minimal.xmind --node "path:/Q2/Payment" --json
```

Preview a subtree edit before writing:

```bash
cp tests/fixtures/xmind/minimal.xmind /tmp/roadmap.xmind
xmind add-tree /tmp/roadmap.xmind \
  --parent "path:/Q2" \
  --input docs/examples/simple-tree.yaml \
  --dry-run \
  --json
```

Apply only after the dry-run output matches the intended change:

```bash
xmind add-tree /tmp/roadmap.xmind \
  --parent "path:/Q2" \
  --input docs/examples/simple-tree.yaml \
  --apply \
  --backup \
  --json
xmind validate /tmp/roadmap.xmind --json
```

## Safety Model

- Mutating commands require exactly one of `--dry-run` or `--apply`.
- Dry runs compute planned changes without changing the filesystem.
- Apply paths validate the workbook before replacing the original file.
- `--backup` creates a timestamped copy for in-place workbook mutations.
- Ambiguous selectors fail instead of guessing.
- JSON output follows documented success and error envelope contracts.

See [mutation semantics](docs/reference/mutation-semantics.md),
[output formats](docs/reference/output-formats.md), and the
[agent error contract](docs/reference/agent-error-contract.md) for details.

## Documentation

- [Documentation map](docs/README.md)
- [Installation](docs/installation.md)
- [Command reference](docs/reference/cli-overview.md)
- [Quick start guide](docs/guides/quick-start.md)
- [Agent recipes](docs/guides/agent-recipes.md)
- [Safe editing workflow](docs/guides/safe-editing-workflow.md)
- [Mutation semantics](docs/reference/mutation-semantics.md)
- [Release policy](docs/technical/release-policy.md)
- [Changelog](CHANGELOG.md)

## Quality Checks

Run the local quality gate before sharing changes:

```bash
./scripts/quality-gate.sh
```

Focused documentation checks can be run with:

```bash
cargo test --test doc_examples_test
git diff --check
```

The CI workflow covers formatting, clippy, tests, docs build, release build
smoke checks, `cargo audit`, and `cargo deny`.

[![CI](https://github.com/ivan-94/xmind-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/ivan-94/xmind-cli/actions/workflows/ci.yml)

## Contributing And Support

This repository is still actively evolving, so small, focused changes are easier
to review than broad rewrites. When reporting an issue, include the command you
ran, whether `--json` was used, the exit status, and a minimal workbook or
fixture path when possible.

Before changing command behavior, update the relevant docs and tests together so
README examples, command references, and CLI behavior stay aligned.

## License

`Cargo.toml` declares this project as MIT licensed.

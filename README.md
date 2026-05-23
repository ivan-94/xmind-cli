# xmind-cli

## Source Manifest

### Sources

- GitHub PRD: <https://github.com/ivan-94/xmind-cli/issues/1>
- GitHub issue: <https://github.com/ivan-94/xmind-cli/issues/2>
- `PLAN.md`: Phase 18 GitHub bootstrap, repository positioning, install channels, and release/E2E scope.
- `docs/prd/1/implementation-notes.html`: PRD #1 slice workflow, current decisions, and verification baseline.
- `docs/README.md`: documentation map and product posture.
- `docs/product/vision.md`: agent and human workflow positioning.
- `docs/product/agent-friendly-cli.md`: agent-native CLI requirements.
- `docs/reference/cli-overview.md`: command groups and CLI contract.
- `docs/reference/mutation-semantics.md`: dry-run, apply, backup, and safety rules.
- `docs/reference/output-formats.md`: JSON envelope and human output contracts.
- `docs/installation.md`: current local install and release build instructions.
- `docs/technical/e2e-test-plan.md`: user-perspective E2E plan and release smoke scope.
- `Cargo.toml`: package name, executable name, version, description, and MIT license metadata.

### Produced artifacts

- `README.md`

### Key decisions

- Position the repository as the public GitHub entrypoint for `ivan-94/xmind-cli` while the executable remains `xmind`.
- State that this is an unofficial XMind CLI and is not affiliated with XMind.
- Describe planned release channels without presenting unreleased binaries, scripts, or Homebrew formulae as available.

### Verification evidence

- README baseline is covered by `tests/cli/doc_examples_test.rs`.

### Open questions / risks

- A standalone `LICENSE` file is not present in this slice; `Cargo.toml` currently declares `MIT`.
- GitHub Release binaries, install script, and Homebrew tap are planned by PRD #1 but are not available yet.

## Overview

`xmind-cli` is an unofficial command line interface for inspecting, querying, editing, validating, and exporting XMind workbooks. The repository name is `xmind-cli`; the installed executable is `xmind`.

The project is built for AI-assisted and human workflows where a mind map needs to behave like structured data instead of an opaque zip package. Agents can inspect a workbook, locate topics with stable selectors, preview mutations with JSON diffs, apply changes explicitly, and validate the result before handing work back to a human.

## Repository Metadata

Suggested GitHub description:

```text
Agent-native CLI for inspecting and editing XMind workbooks
```

Suggested topics:

```text
xmind, cli, rust, mindmap, agents, automation, json, productivity
```

Remote repository:

```text
git@github.com:ivan-94/xmind-cli.git
```

## Status

This repository is in pre-release hardening for PRD #1. Local source builds, CLI integration tests, and documentation contract checks exist today. Public release artifacts, install script automation, Homebrew publication, and expanded E2E fixtures are tracked as follow-up issues under PRD #1.

The project is unofficial and is not endorsed by or affiliated with XMind.

## Install Matrix

| Channel | Status | Command or location |
| --- | --- | --- |
| Cargo source install | Available today | `cargo install --path .` |
| Local release build | Available today | `cargo build --workspace --release`, then run `target/release/xmind` |
| GitHub release binaries | Planned | Will be published from GitHub Releases after release automation lands. |
| Install script | Planned | Will be documented after the script is added and release artifacts are stable. |
| Homebrew tap | Planned | Expected path is `ivan-94/homebrew-tap`; formula publication is a separate follow-up. |

See [docs/installation.md](docs/installation.md) for current local install, release build, shell completion, and verification commands.

## Quick Start

Build or install the binary, then inspect a fixture:

```bash
cargo install --path .
xmind tree tests/fixtures/xmind/minimal.xmind --depth 2 --json
```

Common read workflow:

```bash
xmind inspect roadmap.xmind --json
xmind sheets roadmap.xmind --json
xmind find roadmap.xmind --title "Payment" --json
xmind get roadmap.xmind --node "path:/Q2/Payment" --json
```

Common safe edit preview:

```bash
xmind add-tree roadmap.xmind \
  --parent "path:/Q2" \
  --input docs/examples/simple-tree.yaml \
  --dry-run \
  --json
```

`add-tree --apply` is planned as part of PRD #1 issue #18. Until that lands, treat
`add-tree` as a preview-only example in this README and use implemented mutation
commands for applied workbook edits.

## Safety Model

- Read commands support machine-readable JSON for agent continuation.
- Mutating workbook commands require exactly one of `--dry-run` or `--apply`.
- Dry runs compute planned changes without changing the filesystem.
- Apply paths are expected to validate before replacing the original workbook.
- `--backup` creates a timestamped copy for in-place workbook mutations.
- Ambiguous selectors fail instead of guessing.

See [docs/reference/mutation-semantics.md](docs/reference/mutation-semantics.md), [docs/reference/output-formats.md](docs/reference/output-formats.md), and [docs/reference/agent-error-contract.md](docs/reference/agent-error-contract.md) for the detailed contracts.

## Documentation

- [Documentation map](docs/README.md)
- [Command reference](docs/reference/cli-overview.md)
- [Install and release build notes](docs/installation.md)
- [Agent workflows](docs/guides/agent-recipes.md)
- [Safe editing workflow](docs/guides/safe-editing-workflow.md)
- [E2E test plan](docs/technical/e2e-test-plan.md)
- [Implementation backlog](PLAN.md)

## Quality Checks

Run the local quality gate before sharing changes:

```bash
./scripts/quality-gate.sh
```

The current CI workflow runs formatting, clippy, tests, docs build, a release build smoke check, cargo audit, and cargo deny.

[![CI](https://github.com/ivan-94/xmind-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/ivan-94/xmind-cli/actions/workflows/ci.yml)

## License

`Cargo.toml` declares this project as MIT licensed.

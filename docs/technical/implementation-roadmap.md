# Implementation Roadmap

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: Phased implementation plan for the Rust CLI
- Last updated: 2026-05-21

## Phase 0: Rust Project Foundation

Deliver:

- `Cargo.toml`
- `rust-toolchain.toml`
- `rustfmt.toml`
- `clippy.toml`
- initial `src/main.rs`
- CI or local gate script

Quality gate:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Phase 1: Domain Core

Deliver:

- `Workbook`, `Sheet`, `Topic`,
- `TopicPath`,
- `Selector`,
- `Position`,
- typed errors,
- JSON envelope DTOs.

Tests:

- path escaping,
- selector parsing,
- position parsing,
- error envelope serialization.

## Phase 2: Read-Only XMind Support

Deliver:

- package reader,
- modern XMind format detection,
- `inspect`,
- `sheets`,
- `tree`,
- `get`,
- `find` exact title.

Tests:

- fixture read tests,
- JSON snapshot tests,
- ambiguous selector tests.

## Phase 3: Query, Fields, and Compact Output

Deliver:

- query parser,
- query evaluator,
- `--fields`,
- `--format compact-json`,
- `find --query`,
- `find --title-contains`.

Tests:

- query parser precedence,
- legal field validation,
- compact JSON snapshots.

## Phase 4: Safe Single-Topic Mutations

Deliver:

- transactional writer,
- backup writer,
- validation service,
- `add`,
- `set`,
- `delete`,
- `move`,
- `copy`,
- dry-run diffs.

Tests:

- no write on dry-run,
- atomic validation failure,
- root operation rejection,
- delete modes,
- copy id regeneration.

## Phase 5: Tree Input and Patch Engine

Deliver:

- YAML/JSON tree input,
- Markdown heading/list parser,
- patch parser,
- op alias normalization,
- `add-tree`,
- `patch`,
- `merge_tree`,
- `ensure_path`.

Tests:

- patch operation snapshots,
- idempotent merge,
- operation-indexed errors,
- create-missing-path defaults.

## Phase 6: Import, Export, Assets

Deliver:

- `export` JSON/Markdown/outline/text/assets,
- `import` new workbook and into existing workbook,
- topic image attach/remove,
- asset export,
- preservation tests.

Tests:

- markdown round trip,
- asset preservation,
- unsupported asset errors,
- overwrite behavior.

## Phase 7: Release Hardening

Deliver:

- shell completions,
- manpage or generated help snapshots,
- `cargo audit`,
- `cargo deny`,
- release builds,
- installation documentation.

Tests:

- full CLI snapshot suite,
- release-mode test run,
- package smoke test.

## Prioritization Rule

Do not implement broad XMind visual features before the read/patch/write safety loop is fully working. The minimum useful agent-native loop is:

```bash
xmind tree file.xmind --json --depth 2
xmind patch file.xmind --ops ops.yaml --dry-run --json
xmind patch file.xmind --ops ops.yaml --apply --backup --validate-after --json
```


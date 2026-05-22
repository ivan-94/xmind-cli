# XMind CLI Implementation Plan

## Source Manifest

- Sources:
  - `docs/README.md`
  - `docs/product/*.md`
  - `docs/concepts/*.md`
  - `docs/reference/*.md`
  - `docs/reference/commands/*.md`
  - `docs/schemas/*.md`
  - `docs/technical/*.md`
- Scope: Full implementation todo plan for the Rust-based AI-native XMind CLI
- Last updated: 2026-05-21

## Goal

Implement a Rust CLI named `xmind` that satisfies the documented product, command, schema, and technical contracts:

- agent-native JSON envelope,
- explicit `--dry-run` / `--apply`,
- safe XMind read/write with preservation,
- selector/query/path support,
- patch-based batch mutation,
- Markdown/YAML/JSON import paths,
- strong Rust quality gates,
- complete tests and fixtures.

## Phase 0: Repository Foundation

- [x] Create `Cargo.toml`.
- [x] Set package name, binary name, edition, license, and metadata.
- [x] Add `rust-toolchain.toml`.
- [x] Add `rustfmt.toml`.
- [x] Add `clippy.toml`.
- [x] Add initial `deny.toml`.
- [x] Add `.gitignore` for Rust build artifacts.
- [x] Create `src/main.rs`.
- [x] Create initial module tree under `src/`.
- [x] Add `tests/fixtures/` directory.
- [x] Add `tests/cli/` directory.
- [x] Add local quality gate script or documented command.
- [x] Verify `cargo fmt --all -- --check`.
- [x] Verify `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [x] Verify `cargo test --workspace --all-features`.

## Phase 1: CLI Skeleton

- [x] Add `clap` dependency.
- [x] Define top-level `xmind` CLI.
- [x] Implement subcommand enum:
  - [x] `inspect`
  - [x] `sheets`
  - [x] `tree`
  - [x] `find`
  - [x] `get`
  - [x] `add`
  - [x] `add-tree`
  - [x] `set`
  - [x] `delete`
  - [x] `move`
  - [x] `copy`
  - [x] `patch`
  - [x] `diff`
  - [x] `validate`
  - [x] `export`
  - [x] `import`
  - [x] `backup`
  - [x] `restore`
- [x] Implement global options:
  - [x] `--json`
  - [x] `--format`
  - [x] `--fields`
  - [x] `--quiet`
  - [x] `--no-color`
  - [x] `--sheet`
  - [x] `--sheet-id`
  - [x] `--sheet-index`
- [x] Enforce `--dry-run | --apply` required group for mutating commands.
- [x] Ensure invalid CLI usage maps to `invalid_usage`.
- [x] Add help snapshot tests for top-level command.
- [x] Add help snapshot tests for each subcommand.

## Phase 2: Output and Error Foundation

- [x] Add typed `CommandEnvelope<T>`.
- [x] Add typed `CliErrorBody`.
- [x] Add typed `CliWarning`.
- [x] Add `ErrorCode` enum with snake_case serialization.
- [x] Add exit code mapping.
- [x] Implement JSON renderer.
- [x] Implement human text renderer scaffold.
- [x] Implement `--quiet` behavior for human output.
- [x] Implement `--no-color` behavior for human output.
- [x] Add snapshot tests for success JSON envelope.
- [x] Add snapshot tests for failure JSON envelope.
- [x] Add tests for each documented exit code mapping.

## Phase 3: Domain Core

- [x] Implement `Workbook`.
- [x] Implement `Sheet`.
- [x] Implement `Topic`.
- [x] Implement `TopicId`.
- [x] Implement `SheetId`.
- [x] Implement `AssetId`.
- [x] Implement `TopicImageRef`.
- [x] Implement `ResourceIndex`.
- [x] Implement `PreservationBag`.
- [x] Implement `TopicPath`.
- [x] Implement path parse/render.
- [x] Implement path escaping for `/`.
- [x] Implement path escaping for backslash.
- [x] Implement root path `/`.
- [x] Add property tests for path round trip.
- [x] Add tests for title `/` path segment.
- [x] Add tests proving canonical paths exclude root topic title.

## Phase 4: Selector and Query Engine

- [x] Implement `Selector` enum:
  - [x] `root`
  - [x] `id:`
  - [x] `path:`
  - [x] `title:`
  - [x] `query:`
- [x] Implement selector parser.
- [x] Implement selector render for diagnostics.
- [x] Implement sheet selection:
  - [x] by title,
  - [x] by id,
  - [x] by index.
- [x] Implement selector resolution against a sheet.
- [x] Implement ambiguous selector candidates.
- [x] Implement `not_found` diagnostics.
- [x] Implement query lexer/parser.
- [x] Implement query AST.
- [x] Implement query operators:
  - [x] `=`
  - [x] `!=`
  - [x] `>`
  - [x] `>=`
  - [x] `<`
  - [x] `<=`
  - [x] `contains`
  - [x] `starts_with`
  - [x] `ends_with`
  - [x] `in`
  - [x] `exists`
- [x] Implement query precedence:
  - [x] parentheses,
  - [x] `not`,
  - [x] `and`,
  - [x] `or`.
- [x] Implement query string escaping.
- [x] Add unit tests for selector parsing.
- [x] Add unit tests for query parsing.
- [x] Add evaluator tests for all query fields.

## Phase 5: XMind Package Reader

- [x] Add zip package reader.
- [x] Detect supported modern XMind package format.
- [x] Return `unsupported_format` for unsupported variants.
- [x] Decode workbook JSON into storage DTOs.
- [x] Convert storage DTOs into domain model.
- [x] Preserve unknown package entries.
- [x] Preserve unknown JSON fields.
- [x] Load sheets.
- [x] Load root topics.
- [x] Load child topics.
- [x] Load topic title.
- [x] Load topic notes.
- [x] Load labels.
- [x] Load markers.
- [x] Load hyperlinks.
- [x] Load topic image references where supported.
- [x] Load resource metadata.
- [x] Add minimal `.xmind` fixture.
- [x] Add multiple-sheet `.xmind` fixture.
- [x] Add metadata `.xmind` fixture.
- [x] Add image `.xmind` fixture.
- [x] Add read fixture tests.

## Phase 6: Read Commands

- [x] Implement `inspect`.
- [x] Implement `sheets`.
- [x] Implement `tree`.
- [x] Implement `get`.
- [x] Implement `find --title` exact case-sensitive match.
- [x] Implement `find --title-contains`.
- [x] Implement `find --contains`.
- [x] Implement `find --query`.
- [x] Implement `--depth`.
- [x] Implement `--limit`.
- [x] Implement `--offset`.
- [x] Implement `--fields` validation.
- [x] Implement `--format compact-json` payload shaping:
  - [x] `find` match fields.
  - [x] `tree` topic fields.
  - [x] `get` topic fields.
  - [x] `sheets` sheet fields.
  - [x] `inspect` workbook fields.
- [x] Implement `--include-assets`.
- [x] Add CLI snapshot tests for each read command.
- [x] Add ambiguous sheet tests.
- [x] Add ambiguous selector tests.

## Phase 7: Diff Engine

- [x] Implement structured `Diff`.
- [x] Implement `DiffEvent::Added`.
- [x] Implement `DiffEvent::Removed`.
- [x] Implement `DiffEvent::Updated`.
- [x] Implement `DiffEvent::Moved`.
- [x] Implement summary counts.
- [x] Implement human outline diff renderer.
- [x] Implement JSON diff renderer.
- [x] Add tests for add diff.
- [x] Add tests for delete diff.
- [x] Add tests for update diff.
- [x] Add tests for move diff.

## Phase 8: Transactional Writer and Validation

- [x] Implement package writer scaffold.
- [x] Encode domain model back into supported XMind storage DTOs.
- [x] Merge preserved unknown JSON fields.
- [x] Reuse preserved package entries.
- [x] Write candidate package to temp file in destination directory.
- [x] Implement candidate validation.
- [x] Implement atomic replace.
- [x] Implement backup writer.
- [x] Implement `--backup-dir`.
- [x] Ensure `--validate-after` failure leaves original file untouched.
- [x] Implement `validate` command.
  - [x] Accept documented `validate --strict` option.
- [x] Add tests for dry-run not writing.
- [x] Add tests for validation failure rollback.
- [x] Add tests for backup creation.

## Phase 9: Single-Topic Mutations

- [x] Implement mutation planning service.
- [x] Implement `add`.
- [x] Implement `set --title`.
- [x] Implement `set --note`.
- [x] Implement `set --append-note`.
- [x] Implement `set --set-labels`.
- [x] Implement `set --add-label`.
- [x] Implement `set --remove-label`.
- [x] Implement `set --set-markers`.
- [x] Implement `set --add-marker`.
- [x] Implement `set --remove-marker`.
- [x] Implement `set --hyperlink`.
- [x] Implement `set --clear` repeated flag semantics.
- [x] Implement `delete`.
- [x] Implement `delete --children-only`.
- [x] Implement `delete --promote-children`.
- [x] Implement `move`.
- [x] Implement `copy`.
- [x] Implement `copy --preserve-ids` guardrails.
- [x] Implement `--position first`.
- [x] Implement `--position last`.
- [x] Implement `--position index:N`.
- [x] Implement `--position before:<selector>`.
- [x] Implement `--position after:<selector>`.
- [x] Implement `--create-missing-path`.
- [x] Implement intermediate topic defaults.
- [x] Reject unsupported root operations.
- [x] Add CLI tests for each mutation.
- [x] Add JSON dry-run snapshots.
- [x] Add applied write tests.

## Phase 10: Tree Input

- [x] Implement YAML tree input parser.
- [x] Implement JSON tree input parser.
- [x] Validate `TopicTree`.
- [x] Support optional input ids for id-based merge.
- [x] Support image fields in tree input.
- [x] Implement Markdown frontmatter parser.
- [x] Implement Markdown heading outline parser.
- [x] Implement Markdown list outline parser.
- [x] Implement Markdown ordered list parser.
- [x] Implement Markdown task list parser.
- [x] Implement Markdown heading/list hybrid parser.
- [x] Implement Markdown note mapping.
- [x] Implement inline metadata parsing or explicit rejection.
- [x] Add tests for all Markdown modes.
- [x] Add invalid Markdown diagnostics tests.

## Phase 11: Patch Engine

- [x] Parse patch YAML.
- [x] Parse patch JSON.
- [x] Validate top-level `ops`.
- [ ] Normalize aliases:
  - [x] `delete_tree -> delete`
  - [x] `move_tree -> move`
  - [x] `clone_tree -> copy`
- [x] Implement `assert_exists`.
- [x] Implement `assert_not_exists`.
- [x] Implement patch `add`.
- [x] Implement patch `add_tree`.
- [ ] Implement patch `set`.
- [ ] Implement patch `replace_tree`.
- [ ] Implement patch `merge_tree`.
- [ ] Implement patch `delete`.
- [ ] Implement patch `move`.
- [ ] Implement patch `copy`.
- [ ] Implement patch `ensure_path`.
- [ ] Implement patch `sort_children`.
- [ ] Implement patch `set_tree_metadata`.
- [ ] Implement `children_only`.
- [ ] Implement `promote_children`.
- [ ] Implement `preserve_ids`.
- [ ] Implement `prune`.
- [ ] Implement `match_by: title_path`.
- [ ] Implement `match_by: id`.
- [ ] Implement `match_by: path`.
- [ ] Implement `match_by: title`.
- [ ] Add operation-indexed diagnostics.
- [ ] Add conflict detection.
- [ ] Add all-or-nothing working copy semantics.
- [ ] Add dry-run patch snapshots.
- [ ] Add idempotent patch tests.

## Phase 12: Import and Export

- [ ] Implement `export --format json`.
- [ ] Implement `export --format markdown`.
- [ ] Implement `export --format outline`.
- [ ] Implement `export --format text`.
- [ ] Implement `export --format assets`.
- [ ] Implement `export --output`.
- [ ] Implement `export --overwrite`.
- [ ] Implement `export --json` wrapping payload in envelope.
- [ ] Implement `import --output`.
- [ ] Implement `import --into`.
- [ ] Implement `import --overwrite`.
- [ ] Implement `import --dry-run` no-file behavior.
- [ ] Implement creation diff from empty workbook.
- [ ] Add import/export round-trip tests.
- [ ] Add overwrite behavior tests.

## Phase 13: Assets and Images

- [ ] Detect supported image media types.
- [ ] Implement image checksum.
- [ ] Implement topic image attach.
- [ ] Implement topic image replace.
- [ ] Implement topic image clear.
- [ ] Preserve unrelated assets.
- [ ] Export embedded assets.
- [ ] Return `unsupported_asset_type` for unsupported images.
- [ ] Add image fixture tests.
- [ ] Add asset preservation tests.

## Phase 14: Error Coverage

- [ ] Test `invalid_usage`.
- [x] Test `file_not_found`.
- [x] Test `parse_failed`.
- [x] Test `sheet_not_found`.
- [x] Test `ambiguous_sheet`.
- [x] Test `not_found`.
- [x] Test `ambiguous_selector`.
- [ ] Test `invalid_tree_input`.
- [x] Test `invalid_patch`.
- [ ] Test `patch_conflict`.
- [ ] Test `validation_failed`.
- [ ] Test `write_failed`.
- [ ] Test `unsupported_asset_type`.
- [ ] Test `root_operation_not_allowed`.
- [ ] Verify every error includes:
  - [ ] `code`
  - [ ] `message`
  - [ ] `retryable`
  - [ ] `suggested_fix`
  - [ ] `exit_code`
- [x] Verify selector errors include candidates.
- [x] Verify patch errors include `operation_index`.
- [x] Verify schema errors include `field_path`.

## Phase 15: Documentation Synchronization

- [ ] Ensure `xmind --help` matches command references.
- [ ] Ensure command examples run against fixtures.
- [ ] Add generated or checked CLI help snapshots.
- [ ] Update docs if implementation constraints require contract changes.
- [ ] Keep `docs/reference/commands/*.md` aligned with clap options.
- [ ] Keep `docs/schemas/*.md` aligned with serializable DTOs.
- [ ] Keep `docs/technical/*.md` aligned with implemented modules.

## Phase 16: CI and Release Hardening

- [ ] Add CI workflow.
- [ ] Run `cargo fmt --all -- --check` in CI.
- [ ] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` in CI.
- [ ] Run `cargo test --workspace --all-features` in CI.
- [ ] Run `cargo doc --workspace --no-deps` in CI.
- [ ] Add `cargo audit`.
- [ ] Add `cargo deny check`.
- [ ] Add release build smoke test.
- [ ] Add shell completion generation.
- [ ] Add installation instructions.
- [ ] Add changelog.

## First Useful Vertical Slice

The first implementation slice should prove the whole architecture with the smallest useful loop:

- [x] Rust project foundation.
- [x] JSON envelope and typed errors.
- [x] Minimal XMind fixture reader.
- [x] `tree --json --depth`.
- [x] `patch --dry-run` for `add_tree`.
- [x] Structured diff output.
- [x] Quality gate passing.

Target command loop:

```bash
xmind tree tests/fixtures/xmind/minimal.xmind --json --depth 2
xmind patch tests/fixtures/xmind/minimal.xmind --ops docs/examples/patch-add-tree.yaml --dry-run --json
```

## Definition of Done

- [ ] Every documented command exists.
- [ ] Every documented global option is implemented or explicitly rejected.
- [ ] Every documented JSON envelope field is stable.
- [ ] Every documented error code has a test.
- [ ] Every mutating command supports dry-run and apply.
- [ ] Dry-run never writes files.
- [ ] Applied writes are transactional.
- [ ] Unknown XMind data is preserved across supported edits.
- [ ] `cargo fmt`, `cargo clippy`, `cargo test`, and `cargo doc` pass.
- [ ] Command references and CLI help are synchronized.

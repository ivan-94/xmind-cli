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

- [ ] Create `Cargo.toml`.
- [ ] Set package name, binary name, edition, license, and metadata.
- [ ] Add `rust-toolchain.toml`.
- [ ] Add `rustfmt.toml`.
- [ ] Add `clippy.toml`.
- [ ] Add initial `deny.toml`.
- [ ] Add `.gitignore` for Rust build artifacts.
- [ ] Create `src/main.rs`.
- [ ] Create initial module tree under `src/`.
- [ ] Add `tests/fixtures/` directory.
- [ ] Add `tests/cli/` directory.
- [ ] Add local quality gate script or documented command.
- [ ] Verify `cargo fmt --all -- --check`.
- [ ] Verify `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- [ ] Verify `cargo test --workspace --all-features`.

## Phase 1: CLI Skeleton

- [ ] Add `clap` dependency.
- [ ] Define top-level `xmind` CLI.
- [ ] Implement subcommand enum:
  - [ ] `inspect`
  - [ ] `sheets`
  - [ ] `tree`
  - [ ] `find`
  - [ ] `get`
  - [ ] `add`
  - [ ] `add-tree`
  - [ ] `set`
  - [ ] `delete`
  - [ ] `move`
  - [ ] `copy`
  - [ ] `patch`
  - [ ] `diff`
  - [ ] `validate`
  - [ ] `export`
  - [ ] `import`
  - [ ] `backup`
  - [ ] `restore`
- [ ] Implement global options:
  - [ ] `--json`
  - [ ] `--format`
  - [ ] `--fields`
  - [ ] `--quiet`
  - [ ] `--no-color`
  - [ ] `--sheet`
  - [ ] `--sheet-id`
  - [ ] `--sheet-index`
- [ ] Enforce `--dry-run | --apply` required group for mutating commands.
- [ ] Ensure invalid CLI usage maps to `invalid_usage`.
- [ ] Add help snapshot tests for top-level command.
- [ ] Add help snapshot tests for each subcommand.

## Phase 2: Output and Error Foundation

- [ ] Add typed `CommandEnvelope<T>`.
- [ ] Add typed `CliErrorBody`.
- [ ] Add typed `CliWarning`.
- [ ] Add `ErrorCode` enum with snake_case serialization.
- [ ] Add exit code mapping.
- [ ] Implement JSON renderer.
- [ ] Implement human text renderer scaffold.
- [ ] Implement `--quiet` behavior for human output.
- [ ] Implement `--no-color` behavior for human output.
- [ ] Add snapshot tests for success JSON envelope.
- [ ] Add snapshot tests for failure JSON envelope.
- [ ] Add tests for each documented exit code mapping.

## Phase 3: Domain Core

- [ ] Implement `Workbook`.
- [ ] Implement `Sheet`.
- [ ] Implement `Topic`.
- [ ] Implement `TopicId`.
- [ ] Implement `SheetId`.
- [ ] Implement `AssetId`.
- [ ] Implement `TopicImageRef`.
- [ ] Implement `ResourceIndex`.
- [ ] Implement `PreservationBag`.
- [ ] Implement `TopicPath`.
- [ ] Implement path parse/render.
- [ ] Implement path escaping for `/`.
- [ ] Implement path escaping for backslash.
- [ ] Implement root path `/`.
- [ ] Add property tests for path round trip.
- [ ] Add tests for title `/` path segment.
- [ ] Add tests proving canonical paths exclude root topic title.

## Phase 4: Selector and Query Engine

- [ ] Implement `Selector` enum:
  - [ ] `root`
  - [ ] `id:`
  - [ ] `path:`
  - [ ] `title:`
  - [ ] `query:`
- [ ] Implement selector parser.
- [ ] Implement selector render for diagnostics.
- [ ] Implement sheet selection:
  - [ ] by title,
  - [ ] by id,
  - [ ] by index.
- [ ] Implement selector resolution against a sheet.
- [ ] Implement ambiguous selector candidates.
- [ ] Implement `not_found` diagnostics.
- [ ] Implement query lexer/parser.
- [ ] Implement query AST.
- [ ] Implement query operators:
  - [ ] `=`
  - [ ] `!=`
  - [ ] `>`
  - [ ] `>=`
  - [ ] `<`
  - [ ] `<=`
  - [ ] `contains`
  - [ ] `starts_with`
  - [ ] `ends_with`
  - [ ] `in`
  - [ ] `exists`
- [ ] Implement query precedence:
  - [ ] parentheses,
  - [ ] `not`,
  - [ ] `and`,
  - [ ] `or`.
- [ ] Implement query string escaping.
- [ ] Add unit tests for selector parsing.
- [ ] Add unit tests for query parsing.
- [ ] Add evaluator tests for all query fields.

## Phase 5: XMind Package Reader

- [ ] Add zip package reader.
- [ ] Detect supported modern XMind package format.
- [ ] Return `unsupported_format` for unsupported variants.
- [ ] Decode workbook JSON into storage DTOs.
- [ ] Convert storage DTOs into domain model.
- [ ] Preserve unknown package entries.
- [ ] Preserve unknown JSON fields.
- [ ] Load sheets.
- [ ] Load root topics.
- [ ] Load child topics.
- [ ] Load topic title.
- [ ] Load topic notes.
- [ ] Load labels.
- [ ] Load markers.
- [ ] Load hyperlinks.
- [ ] Load topic image references where supported.
- [ ] Load resource metadata.
- [ ] Add minimal `.xmind` fixture.
- [ ] Add multiple-sheet `.xmind` fixture.
- [ ] Add metadata `.xmind` fixture.
- [ ] Add image `.xmind` fixture.
- [ ] Add read fixture tests.

## Phase 6: Read Commands

- [ ] Implement `inspect`.
- [ ] Implement `sheets`.
- [ ] Implement `tree`.
- [ ] Implement `get`.
- [ ] Implement `find --title` exact case-sensitive match.
- [ ] Implement `find --title-contains`.
- [ ] Implement `find --contains`.
- [ ] Implement `find --query`.
- [ ] Implement `--depth`.
- [ ] Implement `--limit`.
- [ ] Implement `--offset`.
- [ ] Implement `--fields` validation.
- [ ] Implement `--format compact-json` payload shaping.
- [ ] Implement `--include-assets`.
- [ ] Add CLI snapshot tests for each read command.
- [ ] Add ambiguous sheet tests.
- [ ] Add ambiguous selector tests.

## Phase 7: Diff Engine

- [ ] Implement structured `Diff`.
- [ ] Implement `DiffEvent::Added`.
- [ ] Implement `DiffEvent::Removed`.
- [ ] Implement `DiffEvent::Updated`.
- [ ] Implement `DiffEvent::Moved`.
- [ ] Implement summary counts.
- [ ] Implement human outline diff renderer.
- [ ] Implement JSON diff renderer.
- [ ] Add tests for add diff.
- [ ] Add tests for delete diff.
- [ ] Add tests for update diff.
- [ ] Add tests for move diff.

## Phase 8: Transactional Writer and Validation

- [ ] Implement package writer scaffold.
- [ ] Encode domain model back into supported XMind storage DTOs.
- [ ] Merge preserved unknown JSON fields.
- [ ] Reuse preserved package entries.
- [ ] Write candidate package to temp file in destination directory.
- [ ] Implement candidate validation.
- [ ] Implement atomic replace.
- [ ] Implement backup writer.
- [ ] Implement `--backup-dir`.
- [ ] Ensure `--validate-after` failure leaves original file untouched.
- [ ] Implement `validate` command.
- [ ] Add tests for dry-run not writing.
- [ ] Add tests for validation failure rollback.
- [ ] Add tests for backup creation.

## Phase 9: Single-Topic Mutations

- [ ] Implement mutation planning service.
- [ ] Implement `add`.
- [ ] Implement `set --title`.
- [ ] Implement `set --note`.
- [ ] Implement `set --append-note`.
- [ ] Implement `set --set-labels`.
- [ ] Implement `set --add-label`.
- [ ] Implement `set --remove-label`.
- [ ] Implement `set --set-markers`.
- [ ] Implement `set --add-marker`.
- [ ] Implement `set --remove-marker`.
- [ ] Implement `set --hyperlink`.
- [ ] Implement `set --clear` repeated flag semantics.
- [ ] Implement `delete`.
- [ ] Implement `delete --children-only`.
- [ ] Implement `delete --promote-children`.
- [ ] Implement `move`.
- [ ] Implement `copy`.
- [ ] Implement `copy --preserve-ids` guardrails.
- [ ] Implement `--position first`.
- [ ] Implement `--position last`.
- [ ] Implement `--position index:N`.
- [ ] Implement `--position before:<selector>`.
- [ ] Implement `--position after:<selector>`.
- [ ] Implement `--create-missing-path`.
- [ ] Implement intermediate topic defaults.
- [ ] Reject unsupported root operations.
- [ ] Add CLI tests for each mutation.
- [ ] Add JSON dry-run snapshots.
- [ ] Add applied write tests.

## Phase 10: Tree Input

- [ ] Implement YAML tree input parser.
- [ ] Implement JSON tree input parser.
- [ ] Validate `TopicTree`.
- [ ] Support optional input ids for id-based merge.
- [ ] Support image fields in tree input.
- [ ] Implement Markdown frontmatter parser.
- [ ] Implement Markdown heading outline parser.
- [ ] Implement Markdown list outline parser.
- [ ] Implement Markdown ordered list parser.
- [ ] Implement Markdown task list parser.
- [ ] Implement Markdown heading/list hybrid parser.
- [ ] Implement Markdown note mapping.
- [ ] Implement inline metadata parsing or explicit rejection.
- [ ] Add tests for all Markdown modes.
- [ ] Add invalid Markdown diagnostics tests.

## Phase 11: Patch Engine

- [ ] Parse patch YAML.
- [ ] Parse patch JSON.
- [ ] Validate top-level `ops`.
- [ ] Normalize aliases:
  - [ ] `delete_tree -> delete`
  - [ ] `move_tree -> move`
  - [ ] `clone_tree -> copy`
- [ ] Implement `assert_exists`.
- [ ] Implement `assert_not_exists`.
- [ ] Implement patch `add`.
- [ ] Implement patch `add_tree`.
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
- [ ] Test `file_not_found`.
- [ ] Test `parse_failed`.
- [ ] Test `sheet_not_found`.
- [ ] Test `ambiguous_sheet`.
- [ ] Test `not_found`.
- [ ] Test `ambiguous_selector`.
- [ ] Test `invalid_tree_input`.
- [ ] Test `invalid_patch`.
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
- [ ] Verify selector errors include candidates.
- [ ] Verify patch errors include `operation_index`.
- [ ] Verify schema errors include `field_path`.

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

- [ ] Rust project foundation.
- [ ] JSON envelope and typed errors.
- [ ] Minimal XMind fixture reader.
- [ ] `tree --json --depth`.
- [ ] `patch --dry-run` for `add_tree`.
- [ ] Structured diff output.
- [ ] Quality gate passing.

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


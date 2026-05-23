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
- Last updated: 2026-05-23

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
- [x] Normalize aliases:
  - [x] `delete_tree -> delete`
  - [x] `move_tree -> move`
  - [x] `clone_tree -> copy`
- [x] Implement `assert_exists`.
- [x] Implement `assert_not_exists`.
- [x] Implement patch `add`.
- [x] Implement patch `add_tree`.
- [x] Implement patch `set`.
- [x] Refactor first pass for `src/app/mod.rs`: move patch dry-run planning, tree input, and Markdown parsing into focused app modules without changing CLI behavior.
- [x] Refactor second pass for `src/app/mod.rs`: move `set` and mutation renderers into focused app modules so features live with their command-specific code.
- [x] Implement patch `replace_tree`.
- [x] Implement patch `merge_tree`.
- [x] Implement patch `delete`.
- [x] Implement patch `move`.
- [x] Implement patch `copy`.
- [x] Implement patch `ensure_path`.
- [x] Implement patch `sort_children`.
- [x] Implement patch `set_tree_metadata`.
- [x] Implement `children_only`.
- [x] Implement `promote_children`.
- [x] Implement `preserve_ids`.
- [x] Implement `prune`.
- [x] Implement `match_by: title_path`.
- [x] Implement `match_by: id`.
- [x] Implement `match_by: path`.
- [x] Implement `match_by: title`.
- [x] Add operation-indexed diagnostics.
- [x] Add conflict detection.
- [x] Add all-or-nothing working copy semantics.
- [x] Add dry-run patch snapshots.
- [x] Add idempotent patch tests.

## Phase 12: Import and Export

- [x] Implement `export --format json`.
- [x] Implement `export --format markdown`.
- [x] Implement `export --format outline`.
- [x] Implement `export --format text`.
- [x] Implement `export --format assets`.
- [x] Implement `export --output`.
- [x] Implement `export --overwrite`.
- [x] Implement `export --json` wrapping payload in envelope.
- [x] Implement `import --output`.
- [x] Implement `import --into`.
- [x] Implement `import --overwrite`.
- [x] Implement `import --dry-run` no-file behavior.
- [x] Implement creation diff from empty workbook.
- [x] Add import/export round-trip tests.
- [x] Add overwrite behavior tests.

## Phase 13: Assets and Images

- [x] Detect supported image media types.
- [x] Implement image checksum.
- [x] Implement topic image attach.
- [x] Implement topic image replace.
- [x] Implement topic image clear.
- [x] Preserve unrelated assets.
- [x] Export embedded assets.
- [x] Return `unsupported_asset_type` for unsupported images.
- [x] Add image fixture tests.
- [x] Add asset preservation tests.

## Phase 14: Error Coverage

- [x] Test `invalid_usage`.
- [x] Test `file_not_found`.
- [x] Test `parse_failed`.
- [x] Test `sheet_not_found`.
- [x] Test `ambiguous_sheet`.
- [x] Test `not_found`.
- [x] Test `ambiguous_selector`.
- [x] Test `invalid_tree_input`.
- [x] Test `invalid_patch`.
- [x] Test `patch_conflict`.
- [x] Test `validation_failed`.
- [x] Test `write_failed`.
- [x] Test `unsupported_asset_type`.
- [x] Test `root_operation_not_allowed`.
- [x] Verify every error includes:
  - [x] `code`
  - [x] `message`
  - [x] `retryable`
  - [x] `suggested_fix`
  - [x] `exit_code`
- [x] Verify selector errors include candidates.
- [x] Verify patch errors include `operation_index`.
- [x] Verify schema errors include `field_path`.

## Phase 15: Documentation Synchronization

- [x] Ensure `xmind --help` matches command references.
- [x] Ensure command examples run against fixtures.
- [x] Add generated or checked CLI help snapshots.
- [x] Update docs if implementation constraints require contract changes.
- [x] Keep `docs/reference/commands/*.md` aligned with clap options.
- [x] Keep `docs/schemas/*.md` aligned with serializable DTOs.
- [x] Keep `docs/technical/*.md` aligned with implemented modules.

## Phase 16: CI and Release Hardening

- [x] Add CI workflow.
- [x] Run `cargo fmt --all -- --check` in CI.
- [x] Run `cargo clippy --workspace --all-targets --all-features -- -D warnings` in CI.
- [x] Run `cargo test --workspace --all-features` in CI.
- [x] Run `cargo doc --workspace --no-deps` in CI.
- [x] Add `cargo audit`.
- [x] Add `cargo deny check`.
- [x] Add release build smoke test.
- [x] Add shell completion generation.
- [x] Add installation instructions.
- [x] Add changelog.

## Phase 17: Post-Audit Documentation Contract Closure

Source Manifest:

- Sources:
  - `docs/reference/commands/add-tree.md`
  - `docs/reference/commands/patch.md`
  - `docs/reference/commands/diff.md`
  - `docs/reference/commands/validate.md`
  - `docs/reference/commands/import.md`
  - `docs/reference/mutation-semantics.md`
  - `implementation-notes.html`
  - `src/app/mod.rs`
  - `src/app/patch.rs`
  - `src/cli/commands.rs`
- Produced artifacts:
  - `implementation-notes.html`
  - `PLAN.md`
- Key decisions:
  - Treat documented-but-missing behavior as implementation backlog, not as silently accepted drift.
  - Add red tests first for each gap so help/schema-only checks cannot mask behavior-level mismatches.
  - Prefer implementing `import --into --backup` over weakening the mutation backup contract; keep `import --output` without backup.
  - After issues #18 through #22, the Phase 17 command contract gaps are closed in code and default tests.
  - Issue #23 is the final documentation synchronization slice; it does not add new CLI behavior.
- Verification evidence:
  - `tests/e2e/red_contract_gaps_test.rs` now runs all Phase 17 closure tests by default.
  - The rustup toolchain path `PATH=/opt/homebrew/opt/rustup/bin:$PATH` is the working local verification route on this machine.
- Open questions / risks:
  - `diff` intentionally remains a single-workbook validation/no-change surface for this release. Workbook-vs-workbook or planned-operation compare modes require a future documented input surface.
  - Real XMind App fixtures remain human-gated under #11 and are not part of Phase 17 closure.

Audit findings to close:

- [x] Add behavior tests proving the Phase 17 gaps fail before implementation and pass after closure:
  - [x] `add-tree --apply` writes through the documented apply path.
  - [x] `patch --apply` writes transactionally and rolls back on later operation errors.
  - [x] `diff --json` emits the documented `summary` plus `changes` envelope.
  - [x] `validate --strict` reports structural diagnostics and fails on warnings/errors.
  - [x] `import --into --backup` is exposed by clap and preserves workbook safety.
- [x] Implement `add-tree --apply`:
  - [x] Reuse the existing YAML/JSON/Markdown tree input parser and validation.
  - [x] Reuse the same selector resolution and diff planning as dry-run.
  - [x] Write through the transactional XMind writer path.
  - [x] Honor `--backup` and include backup path in JSON results.
  - [x] Add applied-write tests proving dry-run/apply parity and unknown data preservation.
- [x] Implement `patch --apply`:
  - [x] Convert the dry-run working-copy result into an applied workbook write.
  - [x] Preserve operation-indexed diagnostics before any write.
  - [x] Keep patch application all-or-nothing.
  - [x] Honor `--backup` and include backup path in JSON results.
  - [x] Add tests for successful multi-op apply, failed op rollback, backup creation, and dry-run/apply diff parity.
- [x] Close the `diff` command contract:
  - [x] Decide and document the supported input shape for this release.
  - [x] Implement JSON output matching `summary` plus `changes` for the single-workbook surface.
  - [x] Add help, JSON, and human-output tests so `diff` can no longer be a noop.
- [x] Implement real `validate` checks:
  - [x] Detect missing sheets/root topics/required fields where the supported storage model requires them.
  - [x] Detect duplicate topic ids within a workbook.
  - [x] Detect invalid topic ordering or malformed child arrays that the reader can preserve but should warn about.
  - [x] Report warnings and errors with stable paths/sheet context.
  - [x] Make `--strict` fail when warnings are present.
- [x] Resolve `import --into` backup semantics:
  - [x] Add `--backup` support for `import --into`.
  - [x] Keep `import --output` overwrite behavior separate from mutation backup behavior.
  - [x] Add CLI help and behavior tests covering both target modes.
- [x] Strengthen documentation synchronization:
  - [x] Add tests that command references, help surface, docs examples, and E2E coverage docs stay aligned.
  - [x] Add targeted command-reference tests for apply paths and JSON envelopes.
  - [x] Update implementation notes after the closure slices, removing resolved deviations and risks.
- [x] Restore local quality evidence:
  - [x] Route around the local Homebrew Rust `libunwind.1.dylib` issue with the rustup PATH.
  - [x] Run `./scripts/quality-gate.sh` with `PATH=/opt/homebrew/opt/rustup/bin:$PATH`.
  - [x] Keep `cargo doc --workspace --no-deps` covered by the quality gate.
  - [x] Record the passing commands in `implementation-notes.html`.

## Phase 18: GitHub Infrastructure, Release, and E2E Program

Source Manifest:

- Sources:
  - User alignment on 2026-05-23:
    - remote repository is `git@github.com:ivan-94/xmind-cli.git`;
    - repository already exists and is public;
    - use GitHub Issues as the formal tracker;
    - release through GitHub Releases first, not crates.io;
    - use `cargo-dist` / `dist` for release automation;
    - first install channels are GitHub Release binaries, checksums, install script, Homebrew tap, and `cargo install --git` fallback;
    - E2E means user-perspective tests over real `.xmind` files;
    - default E2E uses one Rust integration-test runner, release uses a thin smoke test;
    - fixtures should include real XMind App-created workbooks, with Computer Use acceptable for creation.
  - `docs/technical/e2e-test-plan.md`
  - `docs/agents/issue-tracker.md`
  - `.github/workflows/ci.yml`
  - `Cargo.toml`
  - `CHANGELOG.md`
  - `implementation-notes.html`
- Produced artifacts:
  - `PLAN.md`
  - `docs/technical/e2e-test-plan.md`
  - `docs/agents/issue-tracker.md`
  - `docs/technical/README.md`
  - `docs/README.md`
  - `implementation-notes.html`
- Key decisions:
  - Keep executable name `xmind`, but use `xmind-cli` for repository, package, release artifacts, and Homebrew formula.
  - Use `v*` SemVer tags starting at `v0.1.0`; `Cargo.toml` version must match the tag without `v`.
  - Root `README.md` becomes the GitHub user entrypoint and must identify the project as an unofficial XMind CLI.
  - Branch protection should be reproducible through a script or documented `gh api` commands, but not run automatically in CI.
  - Homebrew tap uses `ivan-94/homebrew-tap`; formula automation can follow after release artifacts are stable.
- Verification evidence:
  - `git remote -v` had no configured remote before this planning slice.
  - `git remote -v` now reports `origin git@github.com:ivan-94/xmind-cli.git` for fetch and push.
  - `.github/workflows/ci.yml` already exists with fmt, clippy, tests, docs, release build smoke, cargo audit, and cargo deny jobs.
  - Current E2E fixture directory already contains real `.xmind` files under `tests/fixtures/xmind/`.
- Open questions / risks:
  - Branch protection requires GitHub permissions and may need manual UI fallback.
  - Homebrew tap automation touches a second repository and should remain a separate issue from first release artifact generation.

GitHub bootstrap:

- [x] Set local `origin` to `git@github.com:ivan-94/xmind-cli.git`.
- [x] Push the initial main branch for PRD #1 delivery.
- [x] Update root `README.md`:
  - [x] project positioning and unofficial XMind disclaimer;
  - [x] install channels;
  - [x] quick start;
  - [x] safety model;
  - [x] CI/release badges;
  - [x] supported platforms;
  - [x] docs entrypoints;
  - [x] license.
- [x] Update issue tracking to GitHub Issues:
  - [x] Update `docs/agents/issue-tracker.md`.
  - [x] Create GitHub Issues for Phase 17 and Phase 18 slices.
  - [x] Keep `.scratch/` for temporary local drafts only.
- [x] Add branch protection setup:
  - [x] script or documented `gh api` commands;
  - [x] require PR before merge;
  - [x] require status checks;
  - [x] require branch up to date before merge;
  - [x] disallow force pushes and deletions;
  - [x] document UI fallback when API permissions are missing.

Merge-gate CI:

- [x] Split or name required jobs clearly enough for branch protection:
  - [x] format;
  - [x] clippy;
  - [x] unit/integration tests;
  - [x] docs build;
  - [x] release build smoke;
  - [x] security checks;
  - [x] default PR E2E subset.
- [x] Keep full E2E matrix out of required PR checks until stable.
- [x] Add CI documentation explaining PR gate vs release/nightly gate.

Release automation:

- [x] Evaluate and initialize `cargo-dist` / `dist` for this Rust CLI.
- [x] Configure GitHub Release artifacts for:
  - [x] `x86_64-unknown-linux-gnu`;
  - [x] `aarch64-unknown-linux-gnu`;
  - [x] `x86_64-apple-darwin`;
  - [x] `aarch64-apple-darwin`;
  - [x] `x86_64-pc-windows-msvc`.
- [x] Generate checksums and installer script.
- [x] Add release smoke:
  - [x] `xmind --version`;
  - [x] `xmind tree tests/fixtures/xmind/minimal.xmind --json`;
  - [x] `xmind validate tests/fixtures/xmind/minimal.xmind --json`.
- [x] Enforce release version rules:
  - [x] tag format `v*`;
  - [x] `Cargo.toml` version equals tag without `v`;
  - [x] `CHANGELOG.md` moves `Unreleased` content into `## vX.Y.Z - YYYY-MM-DD`.
- [x] Keep crates.io publish out of the first release.

Homebrew and installation:

- [ ] Create or update `ivan-94/homebrew-tap`.
- [ ] Add `xmind-cli` formula installing executable `xmind`.
- [ ] Formula downloads macOS release tarball and verifies SHA256.
- [x] Document planned `brew install ivan-94/tap/xmind-cli` path without presenting it as available.
- [x] Document manual GitHub Release install.
- [x] Document install script.
- [x] Document `cargo install --git git@github.com:ivan-94/xmind-cli.git` as developer fallback.
- [x] Defer Homebrew tap auto-update until release artifacts are stable.

E2E program:

- [x] Add `docs/technical/e2e-test-plan.md`.
- [x] Create fixture manifest for existing and new `.xmind` fixtures.
- [ ] Generate additional golden fixtures with the real XMind App where possible.
- [x] Keep small PR fixtures in Git; avoid Git LFS in the first version.
- [x] Add default PR E2E subset:
  - [x] one success path per command;
  - [x] representative error family paths;
  - [x] one apply + validate per mutation family;
  - [x] patch multi-op dry-run/apply;
  - [x] import/export round trip;
  - [x] backup/restore.
- [x] Add full E2E matrix inventory for release/nightly:
  - [x] all commands;
  - [x] all user-visible branches;
  - [ ] larger real-world fixture set.
- [x] Add docs example execution for `bash e2e` fenced blocks only.
- [x] Record E2E coverage progress in `docs/technical/e2e-test-plan.md` or an adjacent generated matrix.

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

- [x] Every documented command exists.
- [x] Every documented global option is implemented or explicitly rejected.
- [x] Every documented JSON envelope field is stable.
- [x] Every documented error code has a test.
- [x] Every mutating command supports dry-run and apply.
- [x] Dry-run never writes files.
- [x] Applied writes are transactional.
- [x] Unknown XMind data is preserved across supported edits.
- [x] `cargo fmt`, `cargo clippy`, `cargo test`, and `cargo doc` pass.
- [x] Command references and CLI help are synchronized.

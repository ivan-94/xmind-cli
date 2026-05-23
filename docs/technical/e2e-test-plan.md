# E2E Test Plan

## Source Manifest

### Sources

- User alignment on 2026-05-23:
  - E2E means user-perspective testing.
  - Cover every command and every user-visible behavior branch.
  - Use real `.xmind` files.
  - Use a two-layer fixture library: small PR-gate fixtures and larger release/nightly fixtures.
  - Use one Rust E2E runner for the first version; release jobs run only a thin binary smoke test.
  - Generate golden fixtures from the real XMind App where possible, including via Computer Use.
- `PLAN.md`: implementation backlog and post-audit contract closure plan.
- GitHub issue #10: fixture manifest and governance rules slice.
- `docs/reference/commands/*.md`: command behavior and option contracts.
- `docs/reference/mutation-semantics.md`: dry-run, apply, backup, and transactional write rules.
- `docs/reference/output-formats.md`: JSON envelope and human output contracts.
- `docs/reference/errors.md` and `docs/reference/agent-error-contract.md`: error-code and recovery contracts.
- `tests/fixtures/xmind/*.xmind`: current committed workbook fixtures.
- `tests/cli/*.rs`: current CLI integration tests.

### Produced Artifacts

- `docs/technical/e2e-test-plan.md`
- `tests/fixtures/xmind/manifest.md`
- `PLAN.md`
- `implementation-notes.html`

### Key Decisions

- E2E coverage is defined as a command-by-command user behavior matrix, not source-line or internal branch coverage.
- The first E2E runner stays in Rust integration tests using `assert_cmd::Command::cargo_bin("xmind")`.
- Release jobs build real binaries and run a small smoke suite, but do not duplicate the full E2E matrix.
- Default PR E2E runs a stable subset. Full matrix runs on release/nightly until it is fast and stable enough for PR gating.
- Runnable documentation examples must be explicitly marked as `bash e2e`; ordinary `bash` examples are not auto-executed.
- Golden fixtures should be created/saved by the real XMind App. Synthetic fixtures are allowed only for corrupt or impossible edge cases and must be labeled.

### Verification Evidence

- Current repository contains committed `.xmind` fixtures under
  `tests/fixtures/xmind/`, but the valid fixtures are currently
  `synthetic-generated`; issue #11 records the human-gated path for adding
  XMind App-saved `real-xmind-app` fixtures.
- Current repository already has CLI integration tests under `tests/cli/`.
- No new test command was executed while writing this planning document; this document defines future work.

### Open Questions / Risks

- The exact full matrix will evolve as Phase 17 closes currently missing command behavior (`add-tree --apply`, `patch --apply`, `diff`, and deeper `validate` checks).
- Large real-world fixtures must stay small enough for ordinary Git usage. First version target: each fixture under 1 MB and total E2E fixture set under 10 MB.

## Goals

The E2E suite proves that a user can operate the CLI against real XMind workbooks through the public command surface:

- read and inspect workbooks,
- query sheets and topics,
- preview and apply edits,
- preserve unknown workbook data,
- import and export structured content,
- recover from errors with actionable JSON,
- install and run the release binary at a smoke-test level.

## Non-Goals

- Do not chase 100% source branch coverage through E2E.
- Do not test implementation-only DTO details where a user-visible contract is sufficient.
- Do not use hand-written ZIP structures as golden user fixtures.
- Do not introduce Git LFS in the first version.
- Do not duplicate the full E2E matrix against release artifacts in the first version.

## Fixture Strategy

### Default PR Fixtures

Small real `.xmind` files committed to Git and run on every PR:

- minimal workbook,
- multiple sheets,
- duplicate sheet titles,
- duplicate topic titles,
- notes, labels, markers, and hyperlinks,
- topic images and resources,
- unknown package entries and unknown JSON fields,
- shallow tree suitable for mutation and restore tests.

### Release/Nightly Fixtures

Larger but still Git-friendly workbooks for full matrix runs:

- deeper trees,
- wider trees,
- mixed metadata,
- multiple XMind App save variants where available,
- richer image/resource combinations,
- non-ASCII titles and path escaping cases.

### Synthetic Fixtures

Synthetic fixtures are allowed only when a real XMind App file cannot express the scenario:

- malformed ZIP,
- missing `content.json`,
- invalid JSON,
- duplicate topic IDs,
- intentionally corrupted package entries.

Synthetic fixtures must be stored separately or named clearly so future agents do not treat them as representative user files.

### Fixture Manifest

The committed fixture inventory and governance rules live in
`tests/fixtures/xmind/manifest.md`.

Each fixture should have manifest metadata, either in that single manifest file
or adjacent markdown:

- fixture path,
- source: `xmind-app` or `synthetic`,
- creation method,
- covered behavior,
- default PR gate: yes/no,
- full matrix: yes/no,
- mutation-safe copy strategy,
- privacy/license notes,
- whether it may be regenerated.

## E2E Runner

First version uses one runner:

- Rust integration tests.
- `assert_cmd::Command::cargo_bin("xmind")`.
- Temporary directories for mutating tests.
- Real `.xmind` files copied from fixtures before every write test.
- JSON parsed with `serde_json` and asserted with matchers.
- Human output asserted lightly with snapshots or contains checks.

Release smoke is separate:

- build release binary,
- run `xmind --version`,
- run `xmind tree tests/fixtures/xmind/minimal.xmind --json`,
- run `xmind validate tests/fixtures/xmind/minimal.xmind --json`.

## Assertion Strategy

JSON command assertions:

- envelope fields: `ok`, `command`, `workbook`, `dry_run`, `applied`,
- key result fields,
- stable error code and `suggested_fix`,
- selector candidates where applicable,
- operation index and field path for patch errors,
- dynamic paths and backup names matched with predicates.

Human output assertions:

- non-empty output for successful human commands,
- concise shape and important text,
- snapshots only where output is intentionally stable.

Write command assertions:

- run command,
- re-read workbook with `tree`, `get`, or `inspect`,
- run `validate`,
- assert unknown package entries and relevant metadata survive,
- assert backup/restore behavior when applicable.

File output assertions:

- output path exists,
- overwrite behavior is enforced,
- exported JSON/Markdown/text/assets are parseable or inspectable,
- output content is stable enough for downstream use.

## Command Matrix

Every command needs at least:

- success JSON,
- success human output when meaningful,
- invalid usage,
- missing file or parse failure where applicable,
- sheet selection variants where applicable,
- `--fields` and compact output where applicable.

Read commands:

- `inspect`: supported format, resources, capabilities, malformed/unsupported workbook.
- `sheets`: duplicate titles, field filtering, sheet metadata.
- `tree`: depth, fields, include assets, sheet selection, text output.
- `get`: id/path/title/query selectors, depth, assets, not found, ambiguous selector.
- `find`: exact title, contains, query, limit/offset, no matches.
- `validate`: valid, warnings, strict warning failure, structural errors.
- `diff`: all documented input modes after Phase 17 settles the command contract.

Mutation commands:

- `add`: dry-run, apply, backup, positions, create missing path, parent not found, ambiguous parent.
- `add-tree`: YAML/JSON/Markdown input, dry-run/apply parity, backup, invalid tree input.
- `set`: every editable field, clear fields, image attach/replace/clear, unsupported asset.
- `delete`: subtree, children-only, promote-children, root rejection, backup.
- `move`: positions, cycle rejection, root rejection, backup.
- `copy`: default id regeneration, preserve-id guardrail, root rejection, backup.

Batch and exchange:

- `patch`: every op, every alias, dry-run/apply parity, operation-indexed errors, rollback, backup.
- `import`: output/into, YAML/JSON/Markdown, overwrite, backup for `--into`, no file on dry-run.
- `export`: JSON/Markdown/outline/text/assets, stdout/output, overwrite behavior.

Recovery and shell integration:

- `backup`: default dir, custom dir, JSON output, invalid path.
- `restore`: dry-run/apply, backup-before-restore, latest backup selection, invalid backup.
- `completion`: shell variants, no workbook access, non-JSON stdout.

## CI Gate Strategy

PR required E2E subset:

- one success path per command,
- one representative error path per error family,
- one apply + validate per mutation family,
- patch multi-op dry-run/apply,
- import/export round trip,
- backup/restore.

Full E2E matrix:

- runs on release/nightly,
- includes larger fixture set,
- includes slower branch combinations,
- can graduate into PR required checks once stable.

## Documentation Example Execution

Runnable documentation examples use fenced blocks marked as:

````
```bash e2e
xmind tree tests/fixtures/xmind/minimal.xmind --json
```
````

Rules:

- only `bash e2e` blocks are auto-executed,
- ordinary `bash` blocks remain illustrative,
- examples must use committed fixtures or generated temporary files,
- examples that mutate files must copy fixtures into a temporary directory first.

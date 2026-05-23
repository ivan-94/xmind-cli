# XMind Fixture Manifest

## Source Manifest

### Sources

- GitHub issue #10: `https://github.com/ivan-94/xmind-cli/issues/10`
- Parent PRD #1: `https://github.com/ivan-94/xmind-cli/issues/1`
- `docs/technical/e2e-test-plan.md`: fixture strategy, size targets, and manifest fields
- `tests/fixtures/xmind/README.md`: current fixture inventory and structures
- `tests/fixtures/xmind/*.xmind`: committed workbook fixtures inventoried below
- `~/.agents/docs/agents/workflows.md` and `~/.agents/docs/agents/handoff-policy.md`: persistent artifact Source Manifest requirements

### Produced artifacts

- `tests/fixtures/xmind/manifest.md`
- `tests/fixtures/xmind/README.md`
- `docs/technical/e2e-test-plan.md`
- `docs/prd/1/implementation-notes.html`
- `tests/cli/doc_examples_test.rs`

### Key decisions

- The current valid workbook fixtures are labeled `synthetic-generated` because they are generated from repository JSON fixture content, not saved by the XMind App.
- `real-xmind-app` is reserved for fixtures saved by the XMind App and should be the default for future golden user-representative fixtures.
- Corrupt or impossible edge cases use `synthetic-corrupt` and must not be used as representative user workbooks.

### Verification evidence

- Every committed `.xmind` file under `tests/fixtures/xmind/` is listed in the inventory table.
- Current binary fixture size total is 2,072 bytes; every fixture is below 1 MB and the total set is below 10 MB.

### Open questions / risks

- Issue #11 should add real XMind App golden fixtures. Until then, these synthetic-generated valid workbooks are useful for deterministic CLI behavior tests but are not a substitute for app-saved compatibility coverage.

## Governance Rules

- Every committed `.xmind` fixture under `tests/fixtures/xmind/` must have exactly one inventory row before it is used by E2E or CLI integration tests.
- Use `Source` values consistently:
  - `real-xmind-app`: saved by the XMind App or a verified XMind App automation flow.
  - `synthetic-generated`: valid workbook package generated from repository-owned JSON or code for deterministic behavior coverage.
  - `synthetic-corrupt`: intentionally invalid workbook package for error handling.
- Synthetic fixtures must be either clearly named for the invalid condition, such as `malformed.xmind`, or labeled `synthetic-*` in this manifest so future agents do not treat them as representative user files.
- Default PR-gate fixtures should stay small, deterministic, and free of private or licensed user content.
- Full-matrix fixtures may be larger or slower, but the first-version target is each fixture under 1 MB and total E2E fixture set under 10 MB.
- Mutating tests must copy fixture files into a temporary directory before invoking commands with `--apply`, restore, import `--into`, or any other write path.
- Privacy/license notes must identify whether the fixture is repository-authored, public-domain/public-sample, or requires replacement before public release.
- Regeneration status must state whether the fixture can be regenerated, whether regeneration needs the XMind App, and whether it is intentionally not regenerated.

## Inventory

| Fixture path | Source | Creation method | Covered behavior | PR gate | Full matrix | Mutation-safe copy strategy | Privacy/license notes | Regeneration status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| `tests/fixtures/xmind/minimal.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/minimal-content.json` as a minimal valid package. | Single-sheet read paths, baseline `inspect`, `sheets`, `tree`, `get`, `find`, `validate`, dry-run examples, and shallow mutation tests. | yes | yes | Read tests may use in place; write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content; should be replaced or complemented by a `real-xmind-app` minimal golden fixture in issue #11. |
| `tests/fixtures/xmind/duplicate-titles.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/duplicate-titles-content.json`. | Ambiguous topic title selectors, duplicate topic title resolution, move/copy/patch selector behavior. | yes | yes | Write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content; future real fixture should preserve duplicate title semantics. |
| `tests/fixtures/xmind/multiple-sheets.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/multiple-sheets-content.json`. | Multi-sheet inspection, sheet selection, sheet metadata, and command output that reports workbook sheet structure. | yes | yes | Write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content; future real fixture should preserve two-sheet coverage. |
| `tests/fixtures/xmind/duplicate-sheets.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/duplicate-sheets-content.json`. | Ambiguous sheet selector behavior and duplicate sheet title diagnostics. | yes | yes | Write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content; future real fixture should preserve duplicate sheet titles. |
| `tests/fixtures/xmind/metadata.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/metadata-content.json`. | Notes, labels, markers, hyperlinks, metadata reads, metadata search, and metadata mutation tests. | yes | yes | Write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content; future real fixture should validate XMind App metadata serialization. |
| `tests/fixtures/xmind/topic-image.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/topic-image-content.json` with a `resources/payment.png` package entry. | Topic image references, resource listing, asset export, and image metadata reads. | yes | yes | Write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content and placeholder resource; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content and resource entry; future real fixture should validate XMind App image packaging. |
| `tests/fixtures/xmind/malformed.xmind` | synthetic-corrupt | Hand-written invalid bytes for parser error coverage. | Parse failure, malformed workbook error shape, and recovery messaging. | yes | yes | Never mutate in place; tests should read directly or copy only when asserting path-specific error handling. | Repository-authored invalid binary content; no user data or third-party licensed assets. | Intentionally not regenerated by normal fixture generation; edit only when parse-failure coverage changes. |

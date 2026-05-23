# XMind Fixture Manifest

## Source Manifest

### Sources

- GitHub issue #10: `https://github.com/ivan-94/xmind-cli/issues/10`
- GitHub issue #11: `https://github.com/ivan-94/xmind-cli/issues/11`
- Parent PRD #1: `https://github.com/ivan-94/xmind-cli/issues/1`
- `docs/technical/e2e-test-plan.md`: fixture strategy, size targets, and manifest fields
- `tests/fixtures/xmind/README.md`: current fixture inventory and structures
- `tests/fixtures/xmind/*.xmind`: committed workbook fixtures inventoried below
- Local XMind availability checks on 2026-05-23:
  `/Applications/Xmind.app`, `plutil -p .../Info.plist`,
  `osascript -e 'id of app "Xmind"' -e 'version of app "Xmind"'`,
  `sdef /Applications/Xmind.app`, and Computer Use save attempt.
- `~/.agents/docs/agents/workflows.md` and `~/.agents/docs/agents/handoff-policy.md`: persistent artifact Source Manifest requirements

### Produced artifacts

- `tests/fixtures/xmind/manifest.md`
- `tests/fixtures/xmind/README.md`
- `docs/technical/e2e-test-plan.md`
- `implementation-notes.html`
- `docs/prd/1/implementation-notes.html`
- `tests/cli/doc_examples_test.rs`

### Key decisions

- The current valid workbook fixtures are labeled `synthetic-generated` because they are generated from repository JSON fixture content, not saved by the XMind App.
- `real-xmind-app` is reserved for fixtures saved by the XMind App and should be the default for future golden user-representative fixtures.
- Corrupt or impossible edge cases use `synthetic-corrupt` and must not be used as representative user workbooks.
- No issue #11 fixture is labeled `real-xmind-app` in this slice because the available GUI automation path did not successfully save a file into the assigned worktree.

### Verification evidence

- Every committed `.xmind` file under `tests/fixtures/xmind/` is listed in the inventory table.
- Current binary fixture size total is 2,072 bytes; every fixture is below 1 MB and the total set is below 10 MB.
- `cargo test --test doc_examples_test fixture_manifest_records_real_app_handoff_when_no_real_fixtures_exist` preserves the issue #11 handoff evidence below.

### Open questions / risks

- Issue #11 should add real XMind App golden fixtures. Until then, these synthetic-generated valid workbooks are useful for deterministic CLI behavior tests but are not a substitute for app-saved compatibility coverage.
- Remaining real fixture creation is human-gated: an operator must complete XMind's macOS Save panel and then review the resulting `.xmind` files before they are committed.

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

## Issue #11 Real XMind App Handoff

This slice discovered a local XMind desktop app but did not add real-app
fixtures. Do not label any fixture `real-xmind-app` unless it was actually saved
by the real XMind app or a verified XMind app automation flow, then reviewed for
privacy and size.

### XMind availability evidence

- `/Applications/Xmind.app` exists.
- `plutil -p /Applications/Xmind.app/Contents/Info.plist` reports
  `CFBundleShortVersionString` as `26.02.04171`,
  `CFBundleIdentifier` as `net.xmind.vana.app`, and the `.xmind` document type
  for `org.xmind.openformat.xmind`.
- `osascript -e 'id of app "Xmind"' -e 'version of app "Xmind"'` reported app
  version `26.02.04171`.
- `codesign --verify --deep --strict --verbose=2 /Applications/Xmind.app`
  completed with `/Applications/Xmind.app: valid on disk` and
  `satisfies its Designated Requirement`.
- `sdef /Applications/Xmind.app` returned
  `sdef: couldn't get sdef for /Applications/Xmind.app (error -192)`, so no
  scriptable document-save API was available through AppleScript.

### Automation attempt and blocker

- XMind was already running with user workbooks open, so this slice avoided
  editing those documents.
- A safe attempt copied `tests/fixtures/xmind/minimal.xmind` to
  `.scratch/real-fixture-source/minimal-source.xmind` and opened that copy with
  XMind. XMind refused `tests/fixtures/xmind/minimal.xmind` derived content with
  the in-app error "unable to open file" / "file may be corrupted", which proves
  the synthetic package cannot be converted into a real-app fixture by resaving.
- A new workbook was created through XMind UI, renamed `RealAppMinimal`, and
  XMind's `Command+Shift+S` flow reached the "current device" macOS Save panel.
  The Computer Use path-selection attempt for
  `tests/fixtures/xmind/real-app/` returned to the editor without creating a
  file in the worktree. Because no saved file existed to verify, no
  `real-xmind-app` fixture was committed.

### Manual creation steps

1. In `/Applications/Xmind.app`, create each required privacy-safe workbook:
   minimal workbook, multiple sheets, duplicate sheet titles, duplicate topic
   titles, notes/labels/markers/hyperlinks, images/resources, non-ASCII titles,
   and path escaping cases. Unknown package entries or unknown fields may need a
   synthetic complement if the real app cannot express them.
2. Save each workbook directly under `tests/fixtures/xmind/real-app/` with a
   descriptive lowercase filename, for example
   `real-app-minimal.xmind`.
3. Confirm each workbook opens in XMind after saving and contains no private
   user content or third-party licensed assets.
4. For every new fixture, add one inventory row above with source
   `real-xmind-app`, exact XMind version, creation method, covered behavior,
   PR-gate/full-matrix scope, copy strategy, privacy/license notes, and
   regeneration status.
5. Run the CLI read commands against every new fixture:

   ```bash
   xmind inspect tests/fixtures/xmind/real-app/<fixture>.xmind --json
   xmind tree tests/fixtures/xmind/real-app/<fixture>.xmind --json
   xmind validate tests/fixtures/xmind/real-app/<fixture>.xmind --json
   ```

6. Run the relevant fixture manifest and CLI test subset, then the repository
   quality gate when practical:

   ```bash
   PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo test --test doc_examples_test xmind_fixture_manifest_covers_committed_workbooks_and_governance
   PATH=/opt/homebrew/opt/rustup/bin:$PATH ./scripts/quality-gate.sh
   ```

### Remaining human-gated gaps

- No real XMind app-saved fixture exists in this commit.
- The issue #11 acceptance matrix is still open for all real-app categories.
- Real fixture review must be performed by a human or an agent with reliable
  XMind Save panel control before any fixture is labeled `real-xmind-app`.

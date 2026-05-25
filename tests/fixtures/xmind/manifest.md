# XMind Fixture Manifest

## Source Manifest

### Sources

- GitHub issue #10: `https://github.com/ivan-94/xmind-cli/issues/10`
- GitHub issue #11: `https://github.com/ivan-94/xmind-cli/issues/11`
- Parent PRD #1: `https://github.com/ivan-94/xmind-cli/issues/1`
- `docs/technical/e2e-test-plan.md`: fixture strategy, size targets, and manifest fields
- `tests/fixtures/xmind/README.md`: current fixture inventory and structures
- `tests/fixtures/xmind/**/*.xmind`: committed workbook fixtures inventoried below
- Local XMind availability checks on 2026-05-23:
  `/Applications/Xmind.app`, `plutil -p .../Info.plist`,
  `osascript -e 'id of app "Xmind"' -e 'version of app "Xmind"'`,
  `codesign --verify --deep --strict --verbose=2 /Applications/Xmind.app`,
  `sdef /Applications/Xmind.app`, and the Computer Use save flow.
- User instruction on 2026-05-23: use Computer Use to create the real XMind fixture directly.
- `~/.agents/docs/agents/workflows.md` and `~/.agents/docs/agents/handoff-policy.md`: persistent artifact Source Manifest requirements

### Produced artifacts

- `tests/fixtures/xmind/manifest.md`
- `tests/fixtures/xmind/README.md`
- `tests/fixtures/xmind/real-app/real-app-fixture.xmind`
- `docs/technical/e2e-test-plan.md`
- `implementation-notes.html`
- `docs/prd/1/implementation-notes.html`
- `tests/cli/doc_examples_test.rs`
- `tests/e2e/pr_subset_test.rs`
- `tests/e2e/support.rs`

### Key decisions

- The repository now includes one `real-xmind-app` fixture saved by the XMind App through a verified Computer Use flow.
- Existing deterministic workbook fixtures remain labeled `synthetic-generated` because they are generated from repository JSON fixture content, not saved by the XMind App.
- Corrupt or impossible edge cases use `synthetic-corrupt` and must not be used as representative user workbooks.
- The real app fixture is privacy-safe repository-authored content: root title `Real App Fixture` with XMind default branch topics `分支主题 1` through `分支主题 5`.
- Broader real-app categories such as duplicate sheet titles, embedded images, notes, labels, and path escaping remain future full-matrix expansion, not a blocker for this first PR-gate real fixture.

### Verification evidence

- Every committed `.xmind` file under `tests/fixtures/xmind/` is listed in the inventory table, including nested real-app fixtures.
- Current binary fixture size total is 131,535 bytes; every fixture is below 1 MB and the total set is below 10 MB.
- `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet -- tree tests/fixtures/xmind/real-app/real-app-fixture.xmind --json` decoded root title `Real App Fixture` and five branch topics.
- `PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet -- validate tests/fixtures/xmind/real-app/real-app-fixture.xmind --json` returned `valid: true` with no warnings or errors.
- `cargo test --test doc_examples_test fixture_manifest_records_real_app_fixture_and_followups` guards the real-app evidence and follow-up notes.

### Open questions / risks

- The first real-app fixture proves XMind App save/read compatibility for a simple wider workbook, but the full matrix still benefits from more app-saved variants after the CLI stabilizes.
- Future real-app fixtures must still be reviewed for privacy, license safety, size, and mutation-copy behavior before commit.

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
| `tests/fixtures/xmind/minimal.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/minimal-content.json` as a minimal valid package. | Single-sheet read paths, baseline `inspect`, `sheets`, `tree`, `get`, `find`, `validate`, dry-run examples, and shallow mutation tests. | yes | yes | Read tests may use in place; write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content; complemented by the app-saved `real-app-fixture.xmind`. |
| `tests/fixtures/xmind/duplicate-titles.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/duplicate-titles-content.json`. | Ambiguous topic title selectors, duplicate topic title resolution, move/copy/patch selector behavior. | yes | yes | Write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content; future real fixture may preserve duplicate title semantics. |
| `tests/fixtures/xmind/multiple-sheets.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/multiple-sheets-content.json`. | Multi-sheet inspection, sheet selection, sheet metadata, and command output that reports workbook sheet structure. | yes | yes | Write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content; future real fixture may preserve two-sheet coverage. |
| `tests/fixtures/xmind/duplicate-sheets.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/duplicate-sheets-content.json`. | Ambiguous sheet selector behavior and duplicate sheet title diagnostics. | yes | yes | Write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content; future real fixture may preserve duplicate sheet titles. |
| `tests/fixtures/xmind/metadata.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/metadata-content.json`. | Notes, labels, markers, hyperlinks, metadata reads, metadata search, and metadata mutation tests. | yes | yes | Write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content; future real fixture may validate XMind App metadata serialization. |
| `tests/fixtures/xmind/topic-image.xmind` | synthetic-generated | Generated from `tests/fixtures/xmind/topic-image-content.json` with a `resources/payment.png` package entry. | Topic image references, resource listing, asset export, and image metadata reads. | yes | yes | Write tests must copy to a temporary directory before mutation. | Repository-authored synthetic content and placeholder resource; no user data or third-party licensed assets. | Regenerable from the adjacent JSON content and resource entry; future real fixture may validate XMind App image packaging. |
| `tests/fixtures/xmind/malformed.xmind` | synthetic-corrupt | Hand-written invalid bytes for parser error coverage. | Parse failure, malformed workbook error shape, and recovery messaging. | yes | yes | Never mutate in place; tests should read directly or copy only when asserting path-specific error handling. | Repository-authored invalid binary content; no user data or third-party licensed assets. | Intentionally not regenerated by normal fixture generation; edit only when parse-failure coverage changes. |
| `tests/fixtures/xmind/real-app/real-app-fixture.xmind` | real-xmind-app | Created in `/Applications/Xmind.app` version `26.02.04171` by opening a new workbook, editing the root topic to `Real App Fixture`, adding one XMind default sibling branch with Return, and saving through the macOS Save panel via Computer Use. | Real XMind App package structure, app-generated `content.json` and `content.xml`, thumbnail entry, non-ASCII default branch titles, `tree`, `inspect`, and `validate` compatibility. | yes | yes | Read tests may use in place; mutating tests must copy to a temporary directory before mutation. | Repository-authored content only; no user data or third-party licensed assets. | Regenerate manually or through verified Computer Use in XMind App `26.02.04171` or later; verify with `xmind tree` and `xmind validate` before commit. |

## Issue #11 Real XMind App Fixture Evidence and Follow-ups

Issue #11 now has a committed, repository-safe fixture saved by the real XMind
App. Future additions should follow the same provenance rule: label a fixture
`real-xmind-app` only when it was actually saved by the XMind App or a verified
XMind App automation flow, then reviewed for privacy and size.

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

### Automation evidence

- XMind was already running with user workbooks open, so this flow created and
  saved only the new `未命名 2` workbook and avoided saving user documents.
- A new workbook was created through XMind UI, the root topic was edited to
  `Real App Fixture`, one additional default branch was added, and the workbook
  was saved through XMind's local-device Save panel to
  `tests/fixtures/xmind/real-app/real-app-fixture.xmind`.
- The saved package contains `content.json`, `metadata.json`, `manifest.json`,
  `content.xml`, and `Thumbnails/thumbnail.png`, matching an app-saved XMind
  package shape.

### Verification commands

Installed-binary form:

```bash
xmind inspect tests/fixtures/xmind/real-app/real-app-fixture.xmind --json
xmind tree tests/fixtures/xmind/real-app/real-app-fixture.xmind --json
xmind validate tests/fixtures/xmind/real-app/real-app-fixture.xmind --json
```

Local source-tree form:

```bash
PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet -- inspect tests/fixtures/xmind/real-app/real-app-fixture.xmind --json
PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet -- tree tests/fixtures/xmind/real-app/real-app-fixture.xmind --json
PATH=/opt/homebrew/opt/rustup/bin:$PATH cargo run --quiet -- validate tests/fixtures/xmind/real-app/real-app-fixture.xmind --json
```

### Future full-matrix fixture expansion

1. Add more app-saved fixtures after the CLI stabilizes for multiple sheets,
   duplicate titles, notes/labels/markers/hyperlinks, images/resources,
   non-ASCII path escaping, and deeper trees.
2. Save each workbook directly under `tests/fixtures/xmind/real-app/` with a
   descriptive lowercase filename.
3. Confirm each workbook opens in XMind after saving and contains no private
   user content or third-party licensed assets.
4. Add one inventory row above for every new fixture with source
   `real-xmind-app`, exact XMind version, creation method, covered behavior,
   PR-gate/full-matrix scope, copy strategy, privacy/license notes, and
   regeneration status.
5. Run the CLI read commands and the relevant fixture manifest and E2E tests.

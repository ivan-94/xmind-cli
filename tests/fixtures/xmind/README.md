# XMind Fixtures

See `tests/fixtures/xmind/manifest.md` for the fixture inventory, real-vs-synthetic
labels, PR-gate/full-matrix scope, mutation-safe copy rules, privacy/license
notes, regeneration status, and size policy.

`minimal.xmind` is generated from `minimal-content.json` and contains one sheet:

- Roadmap
  - Q2
    - Payment

`duplicate-titles.xmind` is generated from `duplicate-titles-content.json` and
contains duplicate `Payment` topics under different parents for ambiguity tests.

`multiple-sheets.xmind` is generated from `multiple-sheets-content.json` and
contains `Roadmap` plus `Backlog` sheets for sheet selection tests.

`duplicate-sheets.xmind` is generated from `duplicate-sheets-content.json` and
contains two `Roadmap` sheets for ambiguous sheet selector tests.

`metadata.xmind` is generated from `metadata-content.json` and contains a
plain-text note, labels, markers, a hyperlink, and an image reference on the
`Payment` topic for metadata read/search tests.

`topic-image.xmind` is generated from `topic-image-content.json` and contains a
topic image reference plus a `resources/payment.png` package entry.

## Real XMind App Fixtures

No `real-xmind-app` fixtures are committed yet. Issue #11 discovery found
`/Applications/Xmind.app` version `26.02.04171`, but the app rejected the current
synthetic `minimal.xmind` package and the available GUI automation did not
complete a reliable Save As into this worktree.

Use `tests/fixtures/xmind/manifest.md` as the source of truth for the manual
creation checklist, required acceptance matrix, and verification commands before
adding any fixture under `tests/fixtures/xmind/real-app/`.

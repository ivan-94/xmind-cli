## Implementation notes

在开发过程中，请持续维护仓库根目录下的 `implementation-notes.html`，记录任何用户应该了解的实现与规范之间的差异或解释，包括：

- 设计决策：在规范模糊时所做的选择。
- 偏差：故意偏离规范的地方及其原因。
- 权衡：曾考虑的替代方案及最终选择的理由。
- 未决问题：希望用户确认或修改的任何事项。

## Agent skills

### Source Manifest

Source Manifest is required only for PRDs, issues, HAT artifacts, and explicit handoff documents in this repository. Do not add Source Manifest sections to human-facing README files, overview pages, product docs, command references, or implementation notes unless one of those files is intentionally serving as a PRD, issue, HAT artifact, or explicit handoff document.

### Issue tracker

Issues and PRDs are tracked as local markdown files under `.scratch/<feature-slug>/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Triage uses the default five-role vocabulary: `needs-triage`, `needs-info`, `ready-for-agent`, `ready-for-human`, and `wontfix`. See `docs/agents/triage-labels.md`.

### Domain docs

This is a single-context repo. Read `CONTEXT.md` and `docs/adr/` when present, plus the existing `docs/` product, concept, design, guide, and reference material as task-relevant context. See `docs/agents/domain.md`.

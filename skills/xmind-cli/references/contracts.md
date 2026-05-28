# xmind-cli Contracts

This reference maps agent decisions to the repository contract. Prefer the
listed docs over memory when behavior matters.

## Table of contents

- Command groups
- Safety and mutation contract
- JSON envelope
- Selectors and sheet scope
- Structured inputs
- Error recovery
- Repository documentation map

## Command groups

| Group | Commands | Notes |
| --- | --- | --- |
| Inspection | `inspect`, `sheets`, `tree`, `find`, `get` | Read-only; use first |
| Editing | `add`, `add-tree`, `set`, `delete`, `move`, `copy` | Require dry-run or apply |
| Batch and exchange | `patch`, `diff`, `validate`, `export`, `import` | `import` is mutating; `validate` is read-only |
| Recovery | `backup`, `restore` | `restore` is mutating |
| Shell integration | `completion` | No workbook path required |

Primary doc: `docs/reference/cli-overview.md`.

## Safety and mutation contract

Workbook-mutating commands must receive exactly one of:

```text
--dry-run
--apply
```

Applies to:

- `add`
- `add-tree`
- `set`
- `delete`
- `move`
- `copy`
- `patch`
- `import`
- `restore`

Agent rules:

- Dry run leaves the filesystem unchanged.
- Apply performs the validated write path.
- Use `--backup` for in-place workbook mutations.
- Do not infer a mutation happened unless JSON includes `applied: true`.
- Run `xmind validate <file> --json` after successful writes.

Primary doc: `docs/reference/mutation-semantics.md`.

## JSON envelope

`--json` is the automation contract. The command envelope is distinct from any
payload format.

Success shape:

```json
{
  "ok": true,
  "command": "tree",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "result": {}
}
```

Error shape:

```json
{
  "ok": false,
  "command": "patch",
  "workbook": "roadmap.xmind",
  "error": {
    "code": "ambiguous_selector",
    "message": "Selector matched multiple topics.",
    "retryable": true,
    "suggested_fix": "Retry with one of the candidate ids.",
    "candidates": []
  }
}
```

Payload examples:

```bash
xmind tree roadmap.xmind --json --format compact-json --fields id,path,title --depth 2
xmind export roadmap.xmind --format markdown --json
```

Primary docs:

- `docs/reference/output-formats.md`
- `docs/reference/agent-error-contract.md`
- `docs/reference/errors.md`
- `docs/reference/exit-codes.md`
- `docs/schemas/command-output.schema.md`
- `docs/schemas/error.schema.md`

## Selectors and sheet scope

Common selectors:

| Selector | Example | Best use |
| --- | --- | --- |
| `id:` | `id:topic-123` | Final write targets |
| `path:` | `path:/Q2/Payment` | Human-readable unique paths |
| root path | `path:/` | Read root or add children to root |
| title search | `find --title "Payment"` | Discovery |

Path rules:

- Paths are scoped to a sheet.
- Canonical paths are relative to the selected sheet root.
- Literal slashes in topic titles are escaped as `\/`.
- Duplicate sibling titles can make a path ambiguous.

Use `--sheet`, `--sheet-id`, or `--sheet-index` when a workbook has multiple
sheets.

Primary docs:

- `docs/concepts/path-addressing.md`
- `docs/concepts/query-selectors.md`
- `docs/concepts/selectors.md`
- `docs/schemas/selector.schema.md`

## Structured inputs

Tree input supports:

- `title`
- `note`
- `labels`
- `markers`
- `hyperlink`
- `image`
- `children`

Patch operations use snake_case canonical names. Generate canonical names, not
aliases:

- `add`
- `add_tree`
- `set`
- `replace_tree`
- `merge_tree`
- `delete`
- `move`
- `copy`
- `ensure_path`
- `sort_children`
- `set_tree_metadata`

Read these docs before creating payloads:

- `docs/concepts/tree-input.md`
- `docs/concepts/markdown-outline.md`
- `docs/reference/patch-operations.md`
- `docs/reference/fields.md`
- `docs/schemas/topic-tree.schema.md`
- `docs/schemas/patch.schema.md`

## Error recovery

| Code | Agent recovery |
| --- | --- |
| `ambiguous_selector` | Choose a candidate `id:` and retry |
| `not_found` | Rediscover with `tree` or `find` |
| `invalid_tree_input` | Fix input at `field_path` |
| `invalid_patch` | Fix operation at `operation_index` |
| `patch_conflict` | Recompute patch against current tree |
| `validation_failed` | Stop and inspect diagnostics |
| `unsupported_asset_type` | Convert or remove the asset |
| `root_operation_not_allowed` | Target a child topic or allowed root field |
| `write_failed` | Check filesystem path and permissions |

When `--json` is used, expect the structured envelope on stdout. Human stderr
diagnostics do not replace the JSON contract.

## Repository documentation map

- Task recipes: `docs/guides/agent-recipes.md`
- Safe editing: `docs/guides/safe-editing-workflow.md`
- Idempotent workflows: `docs/guides/idempotent-workflow.md`
- Batch edits: `docs/guides/batch-editing.md`
- Domain model: `docs/concepts/domain-model.md`
- Workbook/sheet/topic model: `docs/concepts/workbook-sheet-topic.md`
- Storage and preservation: `docs/technical/xmind-storage.md`

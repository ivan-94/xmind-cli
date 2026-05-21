# Mutation Semantics

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Required apply, dry-run, backup, and validation behavior for mutating commands
- Last updated: 2026-05-21

## Core Rule

Every command that can modify a workbook or write a workbook output requires exactly one of:

```text
--dry-run
--apply
```

This applies to:

- `add`
- `add-tree`
- `set`
- `delete`
- `move`
- `copy`
- `patch`
- `import`
- `restore`

`backup` is an exception because its only purpose is to create a safe copy. `export --output` writes a non-workbook artifact and does not require `--apply`, but it must not overwrite an existing file unless overwrite behavior is explicitly defined.

## Dry Run

`--dry-run` must:

- parse the workbook,
- resolve selectors,
- validate inputs,
- compute the resulting paths and field changes,
- compute a human-readable and JSON diff,
- run validation if `--validate-after` is present,
- leave the filesystem unchanged.

## Apply

`--apply` must:

- perform the same validation as dry-run,
- write changes only after all preflight checks pass,
- preserve unknown fields by default,
- include a result summary,
- include backup and validation results when requested.

Applied workbook writes must use a write-then-validate-then-atomic-replace flow:

1. Load the original workbook.
2. Build the changed workbook in memory or a temporary file.
3. Run `--validate-after` against the changed workbook when requested.
4. If validation fails, leave the original workbook path untouched.
5. If validation passes, replace the original workbook atomically where the platform supports atomic rename.

This is a product guarantee, not an implementation suggestion. `validation_failed` means the original workbook remains unchanged.

## Backup

For workbook mutations, `--backup` creates a timestamped copy before replacing the original workbook.

Recommended agent pattern:

```bash
xmind patch plan.xmind --ops ops.yaml --dry-run --json
xmind patch plan.xmind --ops ops.yaml --apply --backup --validate-after --json
```

## Validation Failure

If `--validate-after` fails, the original workbook must remain unchanged. The command returns `validation_failed` with diagnostics and, if relevant, the temporary output path.

## Output Path

Commands that support `--output` should write to a new path. In-place mutation should use the original workbook path plus `--apply`.

## Create Missing Path

`--create-missing-path` creates intermediate topics from path segments. For `path:/Q2/Payment`, missing topics are created with:

- `title`: the unescaped path segment text, such as `Q2` or `Payment`,
- `id`: a newly generated topic id,
- `note`: empty,
- `labels`: empty,
- `markers`: empty,
- `hyperlink`: absent,
- `image`: absent,
- `children`: the next created segment or final requested topic.

Created intermediate topics are included in dry-run diffs and write summaries.

## Root Topic Rules

The selected sheet root is structurally special:

- `set` may edit root fields only when the compatibility matrix marks that field editable. Root title editing is format-limited.
- `delete`, `move`, and `copy` must reject `root` and `path:/`.
- `replace_tree` must reject root unless a future command explicitly supports replacing an entire sheet tree.
- `add` and `add_tree` may use root as `--parent`.

Root operation failures use `root_operation_not_allowed` with `retryable: false` unless the suggested fix is to target a child topic.

## Agent Notes

Agents should never infer that a mutation happened unless the JSON result includes `applied: true`.

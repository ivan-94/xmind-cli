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

## Agent Notes

Agents should never infer that a mutation happened unless the JSON result includes `applied: true`.


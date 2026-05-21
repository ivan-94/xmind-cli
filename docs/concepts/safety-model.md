# Safety Model

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Safety behavior shared by write commands
- Last updated: 2026-05-21

## Safety Goals

The CLI should prevent accidental data loss, ambiguous edits, and silent corruption while remaining useful for automated agents.

## Required Safety Rules

### Ambiguous Selectors Fail

Commands that require a single target must fail if a selector matches multiple topics.

### Dry Runs Are Complete

`--dry-run` must perform selector resolution, input validation, diff generation, and optional validation. It should not be a shallow syntax check.

Every workbook-mutating command requires exactly one of `--dry-run` or `--apply`; omitting both is invalid.

### Backups Are Easy

Write commands support:

```bash
--backup
--backup-dir .xmind-backups
```

Backup output includes the backup path.

### Unknown Fields Are Preserved

Default behavior is equivalent to:

```bash
--preserve-unknown
```

### Destructive Operations Are Explicit

`delete`, `delete_tree`, and `replace_tree` should produce clear diffs in dry-run mode. Interactive confirmation is not required for agents, but `--dry-run` examples should be prominent.

### Validation Can Be Required

Commands support:

```bash
--validate-after
```

If validation fails after a mutation, the command should not replace the original workbook.

## Recommended Write Flow

```bash
xmind patch plan.xmind --ops ops.yaml --dry-run --json
xmind patch plan.xmind --ops ops.yaml --apply --backup --validate-after --json
xmind validate plan.xmind --json
```

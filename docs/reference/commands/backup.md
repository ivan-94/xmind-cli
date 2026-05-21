# xmind backup

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for creating backups
- Last updated: 2026-05-21

## Purpose

Create a timestamped backup of a workbook.

## Synopsis

```bash
xmind backup <workbook.xmind> [--backup-dir <dir>] [--json]
```

## Options

- `--backup-dir <dir>`: destination directory.
- `--name <name>`: optional backup name.
- `--json`: emit backup metadata.

## Output

```json
{
  "ok": true,
  "command": "backup",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "result": {
    "backup_path": ".xmind-backups/roadmap.20260521-205900.xmind"
  }
}
```

## Errors

- `file_not_found`
- `write_failed`

## Notes for Agents

Prefer write-command `--backup` for normal edits. Use `backup` directly before risky manual experiments.


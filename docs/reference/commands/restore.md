# xmind restore

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for restoring backups
- Last updated: 2026-05-21

## Purpose

Restore a workbook from a backup.

## Synopsis

```bash
xmind restore <backup.xmind> --output <workbook.xmind> (--dry-run | --apply) [--json]
```

## Options

- `--output <workbook.xmind>`: restore target path.
- `--overwrite`: replace existing output.
- `--dry-run` or `--apply`: exactly one is required.
- `--validate-after`: validate restored workbook.
- `--json`: emit restore metadata.

## Output

```json
{
  "ok": true,
  "command": "restore",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": true,
  "result": {
    "restored_from": ".xmind-backups/roadmap.xmind",
    "output": "roadmap.xmind"
  }
}
```

## Errors

- `file_not_found`
- `write_failed`
- `validation_failed`

## Notes for Agents

Never restore over a user file without `--overwrite` being explicit.

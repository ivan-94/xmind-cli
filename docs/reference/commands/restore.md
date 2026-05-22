# xmind restore

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for restoring backups
- Last updated: 2026-05-22

## Purpose

Restore a workbook from a backup.

## Synopsis

```bash
xmind restore [options] (--dry-run | --apply) <workbook.xmind>
```

## Options

- `--dry-run` or `--apply`: exactly one is required.
- `--backup`: create a backup before applying.
- Global output and sheet options are documented in `../global-options.md`.

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

The current CLI restores in place against the workbook positional argument; there is no separate `--output` or `--overwrite` flag in the help surface.

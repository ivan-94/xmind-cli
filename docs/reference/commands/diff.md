# xmind diff

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for comparing workbooks or planned operations
- Last updated: 2026-05-22

## Purpose

Show structural differences for the workbook in the currently implemented diff surface.

## Synopsis

```bash
xmind diff [options] <workbook.xmind>
```

## Options

- Global output and sheet options are documented in `../global-options.md`.

## Output

```json
{
  "ok": true,
  "command": "diff",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "result": {
    "summary": {
      "added": 2,
      "updated": 1,
      "deleted": 0,
      "moved": 1
    },
    "changes": []
  }
}
```

## Errors

- `file_not_found`
- `parse_failed`
- `invalid_patch`
- `invalid_usage`

## Notes for Agents

Use `patch --dry-run --json` when reviewing generated patch files before applying them.

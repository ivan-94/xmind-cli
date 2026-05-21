# Command Output Schema

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Common JSON envelope for command outputs
- Last updated: 2026-05-21

## Success Envelope

```json
{
  "ok": true,
  "command": "tree",
  "workbook": "roadmap.xmind",
  "result": {}
}
```

## Failure Envelope

```json
{
  "ok": false,
  "command": "tree",
  "workbook": "roadmap.xmind",
  "error": {}
}
```

## Common Fields

| Field | Type | Notes |
| --- | --- | --- |
| `ok` | boolean | Success or failure |
| `command` | string | Command name |
| `workbook` | string | Input workbook path |
| `result` | object | Command-specific success payload |
| `error` | object | Structured error |
| `warnings` | array | Nonfatal warnings |

## Write Result Fields

Write commands should include:

```json
{
  "applied": true,
  "dry_run": false,
  "summary": {
    "added": 0,
    "updated": 0,
    "deleted": 0,
    "moved": 0
  },
  "diff": [],
  "backup_path": ".xmind-backups/file.xmind",
  "validation": {
    "valid": true
  }
}
```

Dry runs must include `applied: false` and `dry_run: true`.

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
  "dry_run": false,
  "applied": false,
  "result": {}
}
```

## Failure Envelope

```json
{
  "ok": false,
  "command": "tree",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "error": {}
}
```

## Common Fields

| Field | Type | Notes |
| --- | --- | --- |
| `ok` | boolean | Success or failure |
| `command` | string | Command name |
| `workbook` | string | Input workbook path |
| `dry_run` | boolean | Whether this invocation was a dry run; defaults to false for read commands |
| `applied` | boolean | Whether this invocation wrote workbook changes; defaults to false for read commands |
| `result` | object | Command-specific success payload |
| `error` | object | Structured error |
| `warnings` | array | Nonfatal warnings |

## Write Result Fields

Write command envelopes put `dry_run` and `applied` at the top level. Write command `result` objects should include:

```json
{
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

Dry runs must use top-level `applied: false` and `dry_run: true`. Applied writes must use top-level `applied: true` and `dry_run: false`.

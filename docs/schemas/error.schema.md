# Error Schema

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Common JSON error schema
- Last updated: 2026-05-22

## Shape

```json
{
  "code": "ambiguous_selector",
  "message": "Selector matched multiple topics.",
  "retryable": true,
  "suggested_fix": "Retry with one candidate id.",
  "selector": "title:\"Payment\"",
  "candidates": [],
  "operation_index": 2,
  "operation": "merge_tree",
  "field_path": null,
  "exit_code": 6,
  "details": {}
}
```

## Fields

| Field | Required | Type | Notes |
| --- | --- | --- | --- |
| `code` | yes | string | Stable machine-readable code |
| `message` | yes | string | Human-readable summary |
| `retryable` | yes | boolean | Whether retry can succeed after changing command input |
| `suggested_fix` | yes | string | Concrete recovery action |
| `selector` | no | string | Failed selector |
| `candidates` | no | array | Candidate topics or sheets |
| `operation_index` | no | number | Patch operation index |
| `operation` | no | string | Canonical patch operation name |
| `field_path` | no | string | Invalid input field path |
| `path` | no | string | Related topic or filesystem path |
| `exit_code` | no | number | Process exit code |
| `details` | no | object | Command-specific data |

## Candidate Topic

```json
{
  "id": "topic-123",
  "path": "/Q2/Payment",
  "title": "Payment",
  "sheet": "Roadmap"
}
```

## Agent Notes

Agents should use `code` and `retryable` to decide recovery behavior. `suggested_fix`, `candidates`, `field_path`, and `operation_index` should be specific enough to retry safely. For `cloud_download_failed`, `details` includes the normalized logical workbook path, the `.icloud` placeholder path, and the attempted materialization commands.

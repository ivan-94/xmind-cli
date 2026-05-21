# Errors

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Error code catalog and structured error behavior
- Last updated: 2026-05-21

## Error Object

```json
{
  "ok": false,
  "command": "get",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": false,
  "error": {
    "code": "ambiguous_selector",
    "message": "Selector matched multiple topics.",
    "retryable": true,
    "suggested_fix": "Retry with one of the candidate ids.",
    "selector": "title:\"Payment\"",
    "candidates": [
      {
        "id": "topic-a",
        "path": "/Q1/Payment",
        "title": "Payment"
      }
    ]
  }
}
```

## Error Codes

| Code | Meaning |
| --- | --- |
| `invalid_usage` | Command arguments are invalid |
| `file_not_found` | Workbook path does not exist |
| `parse_failed` | Workbook cannot be read |
| `sheet_not_found` | Sheet selector did not match |
| `ambiguous_sheet` | Sheet selector matched multiple sheets |
| `not_found` | Node selector did not match |
| `ambiguous_selector` | Node selector matched multiple topics |
| `invalid_tree_input` | Tree input failed validation |
| `invalid_patch` | Patch file failed validation |
| `patch_conflict` | Patch operations conflict |
| `validation_failed` | Workbook validation failed |
| `write_failed` | Output file could not be written |
| `unsupported_asset_type` | Asset type is not supported for the requested operation |
| `root_operation_not_allowed` | Requested operation cannot be applied to the selected sheet root |

## Error Design

Errors should tell the caller what to do next. For selectors, include candidates. For schema failures, include field paths. For patch failures, include `operation_index`. For validation failures, include affected sheet and topic paths when possible. The full agent contract is documented in `agent-error-contract.md`.

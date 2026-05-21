# Agent Error Contract

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Required structured error fields and recovery guidance for agents
- Last updated: 2026-05-21

## Purpose

Errors must be actionable. An agent should be able to decide whether to retry, ask for clarification, choose a candidate, or stop.

## Required Error Envelope

```json
{
  "ok": false,
  "command": "patch",
  "workbook": "roadmap.xmind",
  "error": {
    "code": "ambiguous_selector",
    "message": "Selector matched multiple topics.",
    "retryable": true,
    "suggested_fix": "Retry with one of the candidate ids.",
    "selector": "title:\"Payment\"",
    "operation_index": 2,
    "candidates": []
  }
}
```

## Required Error Fields

| Field | Required | Meaning |
| --- | --- | --- |
| `code` | yes | Stable machine-readable error code |
| `message` | yes | Short human-readable summary |
| `retryable` | yes | Whether retry can succeed without changing the workbook manually |
| `suggested_fix` | yes | Concrete next action |
| `details` | no | Command-specific structured details |

## Context Fields

Errors should include these fields when applicable:

- `selector`
- `sheet`
- `path`
- `field_path`
- `operation_index`
- `operation`
- `candidates`
- `exit_code`
- `asset_path`
- `media_type`

## Recovery Patterns

| Code | Agent Recovery |
| --- | --- |
| `ambiguous_selector` | Choose a candidate id and retry |
| `not_found` | Run `tree` or `find` to rediscover paths |
| `invalid_tree_input` | Fix input at `field_path` |
| `invalid_patch` | Fix operation at `operation_index` |
| `patch_conflict` | Recompute patch against current tree |
| `validation_failed` | Do not retry blindly; inspect diagnostics |
| `unsupported_asset_type` | Convert or remove the asset |
| `write_failed` | Check filesystem path and permissions |

## Stream Rule

When `--json` is used, the structured error envelope should be emitted to stdout. Human diagnostics may go to stderr only if they do not replace the JSON envelope.


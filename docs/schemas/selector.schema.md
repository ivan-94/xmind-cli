# Selector Schema

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Documentation schema for selector strings
- Last updated: 2026-05-21

## Selector Grammar

```text
root
id:<topic-id>
path:/<segment>/<segment>
title:"<title>"
query:<expression>
```

## Examples

```text
root
id:topic-123
path:/Roadmap/Q2/Payment
title:"Payment"
query:title contains "Payment" and marker = "priority-1"
```

## Resolution Output

Resolved topics should include:

```json
{
  "id": "topic-123",
  "path": "/Roadmap/Q2/Payment",
  "title": "Payment",
  "sheet": "Roadmap"
}
```

## Error Behavior

- Zero matches: `not_found`
- Multiple matches where one is required: `ambiguous_selector`
- Malformed selector: `invalid_usage`


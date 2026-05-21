# Selectors

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Shared selector model for locating sheets and topics
- Last updated: 2026-05-21

## Purpose

Selectors identify topics without exposing XMind storage details. Every read and write command that accepts a node target uses the same selector behavior.

## Selector Types

### ID Selector

```text
id:topic-123
```

Best for scripts after a discovery step.

### Path Selector

```text
path:/Q2/Payment
```

Best for human-agent collaboration.

### Title Selector

```text
title:"Payment"
```

Best for search-like usage. It may be ambiguous.

### Query Selector

```text
query:title contains "pay" and marker = "priority-1"
```

Best for metadata searches and audits.

The full query grammar is documented in `query-selectors.md`.

### Root Selector

```text
root
path:/
```

Targets the selected sheet root.

Path selectors are relative to the selected sheet root. They do not include the sheet title or root topic title.

## Resolution Rules

1. Resolve sheet scope first.
2. Resolve selector within that sheet scope.
3. If zero matches, fail with `not_found`.
4. If more than one match and the command requires one target, fail with `ambiguous_selector`.
5. Return candidates in JSON errors.

## Escaping

Paths use `/` as a delimiter. Literal slashes in titles should be escaped as `\/`.

Example:

```text
path:/API\/SDK/Auth
```

## Candidate Shape

Ambiguous selector errors return candidates:

```json
{
  "id": "topic-123",
  "path": "/Q2/Payment",
  "title": "Payment",
  "sheet": "Roadmap"
}
```

## Agent Notes

Agents should prefer this order:

1. Use `id:` when already known.
2. Use `path:` when generated from prior `tree` or `find` output.
3. Use `title:` only for discovery.
4. Use `query:` for audits and bulk selection.

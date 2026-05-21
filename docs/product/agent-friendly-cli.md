# Agent-Friendly CLI

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Design requirements that make the CLI native to autonomous agents
- Last updated: 2026-05-21

## What Agent-Friendly Means

An agent-friendly CLI is not only scriptable. It actively reduces the risks that agents face:

- incomplete context,
- ambiguous targets,
- repeated execution,
- long output windows,
- hidden file corruption,
- unstable identifiers,
- and human-review needs.

## Required Traits

### Discoverability

Agents need fast commands to understand the map before changing it:

```bash
xmind inspect map.xmind --json
xmind sheets map.xmind --json
xmind tree map.xmind --sheet "Roadmap" --depth 2 --json
xmind find map.xmind --title "Auth" --json
```

### Stable Addressing

All topic operations accept the shared selector grammar:

```text
id:topic-abc
path:/Roadmap/Q2/Auth
title:"Auth"
query:title contains "Auth" and marker = "priority-1"
```

### Compact Context Windows

Read commands support `--depth`, `--fields`, `--limit`, and `--format compact-json` so an agent can avoid dumping a full workbook when it only needs a branch.

### Previewable Writes

Every mutation supports:

```bash
--dry-run
--diff
--json
```

Dry run output includes:

- resolved targets,
- planned operations,
- created paths,
- updated fields,
- deleted paths,
- warnings,
- and validation status if requested.

### Retry Safety

Batch commands support idempotent options:

```bash
--if-exists error|skip|merge|replace|rename
--match-by id|title|title_path|path
--create-missing-path
```

### Clear Failure Modes

Errors are structured:

```json
{
  "ok": false,
  "error": {
    "code": "ambiguous_selector",
    "message": "Selector matched 2 topics.",
    "selector": "title:\"Auth\"",
    "candidates": [
      { "id": "topic-a", "path": "/Roadmap/Q1/Auth" },
      { "id": "topic-b", "path": "/Roadmap/Q2/Auth" }
    ]
  }
}
```

### Reviewability

For human review, write commands can emit outline diffs:

```diff
 /Roadmap/Q2
+  Payment
+    Checkout
+    Refunds
-  Old payment plan
```

## Agent Notes

The CLI should assume agents are careful but not omniscient. It should provide sharp tools with guardrails: explicit selectors, dry-run by default in examples, deterministic JSON, and no hidden mutation of unrelated workbook content.


# Idempotent Workflow

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Retry-safe agent workflows
- Last updated: 2026-05-21

## Why It Matters

Agents may retry after tool failures, context compaction, or interrupted execution. Batch operations should be expressible so retrying does not duplicate nodes.

## Recommended Pattern

```yaml
ops:
  - op: ensure_path
    path: /Q2

  - op: merge_tree
    target: path:/Q2/Payment
    match_by: title_path
    tree:
      title: Payment
      children:
        - title: Checkout
        - title: Refunds
```

Run:

```bash
xmind patch roadmap.xmind --ops ops.yaml --dry-run --json
xmind patch roadmap.xmind --ops ops.yaml --apply --backup --json
xmind validate roadmap.xmind --json
```

## Duplicate Handling

Use:

```text
if_exists: merge
match_by: title_path
```

Avoid `rename` in agent workflows unless duplicate topics are explicitly desired.

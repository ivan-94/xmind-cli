# Batch Editing

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Batch editing strategy and examples
- Last updated: 2026-05-21

## Preferred Interfaces

Use `add-tree` for one subtree and `patch` for multiple coordinated changes.

## Add One Subtree

```bash
xmind add-tree roadmap.xmind --parent "path:/Q2" --input payment.yaml --dry-run
```

## Apply Multiple Operations

```yaml
ops:
  - op: add_tree
    parent: path:/Q2
    tree:
      title: Payment
      children:
        - title: Checkout
        - title: Refunds

  - op: set
    node: path:/Q2/Payment/Refunds
    fields:
      note: Domestic orders only
```

```bash
xmind patch roadmap.xmind --ops ops.yaml --dry-run --json
```

## Review

Batch commands should show a summary and tree diff before applying.

# Agent Recipes

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Agent-oriented command recipes
- Last updated: 2026-05-21

## Discover Before Editing

```bash
xmind inspect plan.xmind --json
xmind sheets plan.xmind --json
xmind tree plan.xmind --sheet "Roadmap" --depth 3 --json
```

## Resolve a Target

```bash
xmind find plan.xmind --title "支付" --json
xmind get plan.xmind --node "id:topic-123" --json
```

## Insert a Generated Tree

```bash
xmind add-tree plan.xmind \
  --parent "id:topic-123" \
  --input generated-tree.yaml \
  --dry-run \
  --json
```

Then apply:

```bash
xmind add-tree plan.xmind \
  --parent "id:topic-123" \
  --input generated-tree.yaml \
  --apply \
  --backup \
  --json
```

## Apply a Complex Patch

```bash
xmind patch plan.xmind --ops ops.yaml --dry-run --json
xmind patch plan.xmind --ops ops.yaml --apply --backup --json
xmind validate plan.xmind --json
```

## Recover from Ambiguity

If a write command returns `ambiguous_selector`, read `error.candidates`, choose an `id:`, and retry.

# Human Recipes

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Human-oriented workflows for common XMind CLI tasks
- Last updated: 2026-05-21

## Print an Outline

```bash
xmind tree roadmap.xmind --sheet "Roadmap" --depth 4
```

## Add a Topic

```bash
xmind add roadmap.xmind --parent "path:/Q2" --title "Payment" --dry-run
xmind add roadmap.xmind --parent "path:/Q2" --title "Payment" --apply --backup
```

## Rename a Topic

```bash
xmind find roadmap.xmind --title "Payment"
xmind set roadmap.xmind --node "path:/Q2/Payment" --title "Payments" --apply --backup
```

## Export for Review

```bash
xmind export roadmap.xmind --format markdown --output roadmap.md
```

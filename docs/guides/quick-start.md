# Quick Start

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: First-run usage flow for humans and agents
- Last updated: 2026-05-21

## Inspect

```bash
xmind inspect roadmap.xmind --json
xmind sheets roadmap.xmind --json
```

## Read

```bash
xmind tree roadmap.xmind --sheet "Roadmap" --depth 2
xmind find roadmap.xmind --title "Payment" --json
xmind get roadmap.xmind --node "path:/Q2/Payment" --json
```

## Edit Safely

```bash
xmind add roadmap.xmind --parent "path:/Q2" --title "Payment" --dry-run
xmind add roadmap.xmind --parent "path:/Q2" --title "Payment" --apply --backup --validate-after
```

## Batch Edit

```bash
xmind patch roadmap.xmind --ops ops.yaml --dry-run --json
xmind patch roadmap.xmind --ops ops.yaml --apply --backup --validate-after --json
```

## Export

```bash
xmind export roadmap.xmind --format markdown --output roadmap.md
```

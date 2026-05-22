# Safe Editing Workflow

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Recommended safe mutation workflow
- Last updated: 2026-05-21

## Workflow

1. Inspect workbook.
2. Resolve target selector.
3. Dry-run mutation.
4. Review JSON summary and diff.
5. Apply with backup.
6. Validate output.

## Commands

```bash
xmind inspect plan.xmind --json
xmind find plan.xmind --title "Payment" --json
xmind add-tree plan.xmind --parent "id:topic-123" --input payment.yaml --dry-run --json
xmind add-tree plan.xmind --parent "id:topic-123" --input payment.yaml --apply --backup --json
xmind validate plan.xmind --json
```

## Recovery

If the result is wrong:

```bash
xmind restore plan.xmind --apply --backup
xmind validate plan.xmind --json
```

# xmind add-tree

## Source Manifest

- Conversation: XMind CLI product design discussion
- GitHub issue #18: add-tree apply writer
- GitHub issue #23: final documentation synchronization
- Scope: Command reference for adding a subtree
- Last updated: 2026-05-23

## Purpose

Insert a whole subtree under a parent topic.

## Synopsis

```bash
xmind add-tree [options] --parent <selector> --input <tree.yaml|tree.json> (--dry-run | --apply) <workbook.xmind>
xmind add-tree [options] --parent <selector> --from-markdown <outline.md> (--dry-run | --apply) <workbook.xmind>
```

## Options

- `--parent <selector>`: parent topic.
- `--input <file>`: YAML or JSON tree input.
- `--from-markdown <file>`: Markdown outline input.
- `--markdown-mode heading|list|hybrid|auto`: Markdown parsing mode.
- `--dry-run` or `--apply`: exactly one is required.
- `--backup`: create a backup before applying.
- Global output and sheet options are documented in `../global-options.md`.

## Output

```json
{
  "ok": true,
  "command": "add-tree",
  "workbook": "roadmap.xmind",
  "dry_run": false,
  "applied": true,
  "result": {
    "created_root": {
      "id": "topic-payment",
      "path": "/Q2/支付能力"
    },
    "summary": {
      "added": 6,
      "updated": 0,
      "deleted": 0
    },
    "backup_path": ".xmind-backups/roadmap-20260523T120000.xmind"
  }
}
```

## Dry Run Behavior

Dry run validates the whole tree and returns a tree diff. No file is written.
Apply uses the same parse, parent resolution, validation, and planning path as
dry-run, then writes through the transactional workbook writer. When `--backup`
is present, `result.backup_path` points to the backup created before replacement.

## Errors

- `invalid_tree_input`
- `not_found`
- `ambiguous_selector`
- `patch_conflict`
- `write_failed`

## Notes for Agents

This is the preferred command for inserting generated plans, breakdowns, and nested task structures. Markdown input supports headings, lists, ordered lists, task lists, and heading/list hybrids.

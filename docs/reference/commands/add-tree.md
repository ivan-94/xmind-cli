# xmind add-tree

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for adding a subtree
- Last updated: 2026-05-21

## Purpose

Insert a whole subtree under a parent topic.

## Synopsis

```bash
xmind add-tree <workbook.xmind> --parent <selector> --input <tree.yaml|tree.json> (--dry-run | --apply) [options]
xmind add-tree <workbook.xmind> --parent <selector> --from-markdown <outline.md> (--dry-run | --apply) [options]
```

## Options

- `--parent <selector>`: parent topic.
- `--input <file>`: YAML or JSON tree input.
- `--from-markdown <file>`: Markdown outline input.
- `--markdown-mode heading|list|hybrid|auto`: Markdown parsing mode.
- `--position <position>`: insertion position.
- `--if-exists error|skip|merge|replace|rename`: duplicate handling.
- `--match-by id|path|title|title_path`: matching strategy.
- `--create-missing-path`: create missing parent path.
- `--dry-run` or `--apply`: exactly one is required.
- `--backup`, `--validate-after`, `--json`.

## Output

```json
{
  "ok": true,
  "command": "add-tree",
  "result": {
    "created_root": {
      "id": "topic-payment",
      "path": "/Roadmap/Q2/支付能力"
    },
    "summary": {
      "added": 6,
      "updated": 0,
      "deleted": 0
    }
  }
}
```

## Dry Run Behavior

Dry run validates the whole tree and returns a tree diff. No file is written.

## Errors

- `invalid_tree_input`
- `not_found`
- `ambiguous_selector`
- `patch_conflict`
- `write_failed`

## Notes for Agents

This is the preferred command for inserting generated plans, breakdowns, and nested task structures. Markdown input supports headings, lists, ordered lists, task lists, and heading/list hybrids.

# Command Taxonomy

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command grouping and naming rationale
- Last updated: 2026-05-21

## Taxonomy

Commands are grouped by user intent:

| Group | Commands | Purpose |
| --- | --- | --- |
| Inspect | `inspect`, `sheets`, `tree`, `find`, `get` | Understand workbook state |
| Edit | `add`, `add-tree`, `set`, `delete`, `move`, `copy` | Mutate topics and subtrees |
| Batch | `patch`, `diff`, `validate` | Apply and verify structured changes |
| Exchange | `export`, `import` | Convert between XMind and portable formats |
| Recovery | `backup`, `restore` | Protect and recover files |

## Naming Rules

- Use verbs for mutations: `add`, `set`, `delete`, `move`, `copy`.
- Use nouns or inspection verbs for reads: `sheets`, `tree`, `find`, `get`.
- Use `*-tree` when the operation explicitly accepts or affects an entire subtree.
- Avoid exposing storage-specific words in user-facing command names.

## Why `add-tree` Exists

`add --title` is good for one topic. Agents often generate a whole structure. `add-tree` makes that intent explicit and lets the CLI validate and diff the whole subtree as one unit.

## Why `patch` Exists

Many useful changes are multi-step. `patch` gives agents a declarative and reviewable format instead of requiring fragile shell scripts.


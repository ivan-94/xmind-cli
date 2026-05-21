# Naming Conventions

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Naming conventions for commands, options, fields, and schemas
- Last updated: 2026-05-21

## Commands

Commands use lowercase kebab-case:

```text
add-tree
```

## Options

Options use long kebab-case names:

```text
--dry-run
--validate-after
--create-missing-path
```

Short aliases should be rare and only for high-frequency human use.

## JSON Fields

JSON fields use snake_case:

```json
{
  "children_count": 3,
  "created_paths": []
}
```

## Operation Names

Patch operation names use snake_case:

```yaml
op: merge_tree
```

## Selectors

Selector prefixes use lowercase words followed by `:`:

```text
id:topic-123
path:/Root/A
title:"A"
query:title contains "A"
```


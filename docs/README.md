# XMind CLI Documentation

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Documentation-first product shape for an agent-friendly XMind CLI
- Last updated: 2026-05-21

## Purpose

This documentation describes the intended external shape of `xmind`, a CLI for reading, querying, editing, validating, and exporting XMind files in a way that is native to agents and still comfortable for humans.

The CLI treats an XMind file as a structured workbook, not as a zip archive or storage format. Users and agents operate on sheets, topic trees, topics, metadata, relationships, and batch patches through stable commands and machine-readable outputs.

## Documentation Map

- `product/`: product vision, principles, scope, and agent-first design goals.
- `concepts/`: domain concepts shared by all commands, including selectors, query grammar, paths, tree input, Markdown outline mapping, asset handling, compatibility, patch semantics, and safety rules.
- `reference/`: command reference, legal fields, mutation semantics, patch operations, global options, output contracts, errors, and exit codes.
- `schemas/`: documented schemas for tree input, patch operations, selectors, command output, and errors.
- `guides/`: workflows for agents and humans.
- `examples/`: copyable inputs and representative outputs.
- `design/`: command taxonomy, naming rules, compatibility policy, and future ideas.

## Reading Order

For product design review:

1. `product/vision.md`
2. `product/principles.md`
3. `product/agent-friendly-cli.md`
4. `concepts/domain-model.md`
5. `design/command-taxonomy.md`

For implementing or using commands:

1. `reference/cli-overview.md`
2. `reference/global-options.md`
3. `concepts/selectors.md`
4. `concepts/query-selectors.md`
5. `reference/fields.md`
6. `concepts/tree-input.md`
7. `concepts/markdown-outline.md`
8. `concepts/compatibility-matrix.md`
9. `reference/mutation-semantics.md`
10. `reference/patch-operations.md`
11. `reference/commands/*.md`

For agent automation:

1. `guides/agent-recipes.md`
2. `guides/safe-editing-workflow.md`
3. `guides/idempotent-workflow.md`
4. `schemas/*.md`
5. `examples/*`

## Product Posture

`xmind` should behave like a small domain-specific database editor for mind maps:

- Query before editing.
- Never guess when a selector is ambiguous.
- Preview writes before applying them.
- Preserve unknown XMind data by default.
- Emit stable JSON for agents.
- Keep human-readable output concise and diff-oriented.

# Markdown Outline Workflow

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Markdown outline import and export workflow
- Last updated: 2026-05-21

## Export to Markdown

```bash
xmind export roadmap.xmind --format markdown --output roadmap.md
```

## Edit Markdown

```md
# Roadmap

## Q2

### [Payment](<https://example.com/payments>)

Payment scope.
```

## Import as a New Workbook

```bash
xmind import --input roadmap.md --output roadmap.xmind --apply
xmind validate roadmap.xmind --json
```

## Insert Markdown Under an Existing Node

```bash
xmind add-tree roadmap.xmind --parent "path:/Q2" --from-markdown payment.md --dry-run
```

## Rules

- Headings, lists, ordered lists, task lists, and heading/list hybrids define hierarchy.
- Markdown export preserves topic hyperlinks as heading links with angle-bracket destinations.
- Markdown import currently treats heading text as titles and does not round-trip hyperlinks.
- Body text under a heading or list item becomes notes.
- Frontmatter can carry metadata.
- Use `--markdown-mode heading|list|hybrid|auto` when auto-detection is not specific enough.

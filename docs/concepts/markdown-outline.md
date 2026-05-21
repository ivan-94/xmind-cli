# Markdown Outline

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Markdown heading and list outline mapping rules
- Last updated: 2026-05-21

## Purpose

Markdown is the most convenient interchange format for humans and agents. The CLI should support both heading outlines and list outlines so agents can generate compact trees without verbose YAML.

## Supported Forms

### Heading Outline

```md
# 支付能力

Q2 核心交付范围。

## 收银台

### 支付方式选择
### 优惠券抵扣
```

### List Outline

```md
- 支付能力
  - 收银台
    - 支付方式选择
    - 优惠券抵扣
  - 退款
    - 原路退回
    - 部分退款
```

### Ordered List Outline

```md
1. 支付能力
   1. 收银台
   2. 退款
```

### Task List Outline

```md
- [ ] 支付能力
  - [x] 收银台
  - [ ] 退款
```

Task list state maps to metadata, not title text:

```yaml
markers:
  - task-open
```

or:

```yaml
markers:
  - task-done
```

### Heading Plus List Hybrid

```md
# Roadmap

## Q2

- 支付能力
  - 收银台
  - 退款
- 会员能力
```

The heading establishes the current parent. Nested list items become descendants under that parent.

## Notes

Paragraphs directly under a heading or list item become that topic's note until the next sibling or descendant item.

```md
- 支付能力

  Q2 核心交付范围。

  - 收银台
```

Maps to:

```yaml
title: 支付能力
note: Q2 核心交付范围。
children:
  - title: 收银台
```

## Metadata

Frontmatter may define defaults for the root topic:

```md
---
labels: [MVP]
markers: [priority-1]
---

- 支付能力
```

Inline metadata may be supported with a compact attribute suffix:

```md
- 支付能力 {labels: [MVP], markers: [priority-1]}
```

If inline metadata cannot be parsed, the command fails with `invalid_tree_input` and a field path.

## Ambiguous Markdown

Markdown input is invalid when:

- it contains multiple top-level roots and the command requires one root,
- heading levels skip in a way that cannot produce a deterministic parent,
- list indentation is inconsistent,
- a list item has no title after task markers and metadata are stripped,
- metadata cannot be parsed.

## Command Options

Commands that consume Markdown support:

```bash
--from-markdown <file>
--markdown-mode heading|list|hybrid|auto
```

`auto` is the default. It detects heading, list, or hybrid structure and fails when detection is ambiguous.


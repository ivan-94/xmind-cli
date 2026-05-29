# Tree Input

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Tree-shaped input accepted by add-tree, patch, import, and merge operations
- Last updated: 2026-05-21

## Purpose

Tree input lets agents create or update an entire subtree at once. This avoids brittle loops that call `xmind add --title` repeatedly.

## YAML Shape

```yaml
title: 支付能力
note: Q2 核心交付范围
labels:
  - MVP
markers:
  - priority-1
children:
  - title: 收银台
    children:
      - title: 支付方式选择
      - title: 优惠券抵扣
  - title: 退款
    children:
      - title: 原路退回
      - title: 部分退款
```

Topic images may be included by reference to a local file:

```yaml
title: 架构方案
image:
  path: ./architecture.png
  alt: Architecture diagram
```

## JSON Shape

```json
{
  "title": "支付能力",
  "note": "Q2 核心交付范围",
  "labels": ["MVP"],
  "children": [
    { "title": "收银台" },
    { "title": "退款" }
  ]
}
```

## Markdown Outline Shape

```md
# 支付能力

Q2 核心交付范围。

## 收银台

### 支付方式选择
### 优惠券抵扣

## 退款

### 原路退回
### 部分退款
```

Mapping rules:

- Headings become topics.
- Heading hierarchy becomes topic hierarchy.
- Paragraph text under a heading becomes the topic note.
- Frontmatter may define metadata.
- Lists, ordered lists, task lists, and heading/list hybrids are supported by the full Markdown outline contract.

See `markdown-outline.md` for deterministic list mapping rules.

## Supported Fields

- `title`
- `note`
- `labels`
- `markers`
- `image`
- `children`

Future fields may include `style`, `relationships`, `summaries`, and `boundaries`.

## Validation

Tree input is invalid when:

- `title` is missing or empty.
- `children` is not a list.
- `image` is not an object with a valid `path` or `asset_id`.
- metadata fields use the wrong type.
- a requested merge strategy requires unique titles but duplicates exist.

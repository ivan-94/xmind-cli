# xmind-cli Payload Examples

Use these examples when generating input files for `add-tree`, `patch`, and
`import`. Keep generated payloads small enough to review before applying.

## Tree YAML for add-tree

```yaml
title: Payment
note: Q2 delivery scope
labels:
  - MVP
markers:
  - priority-1
children:
  - title: Checkout
    children:
      - title: Payment method selection
      - title: Coupon deduction
  - title: Refunds
    children:
      - title: Original route refund
      - title: Partial refund
```

Run:

```bash
xmind add-tree plan.xmind --parent "id:topic-123" --input payment.yaml --dry-run --json
```

## Tree JSON

```json
{
  "title": "Payment",
  "note": "Q2 delivery scope",
  "labels": ["MVP"],
  "children": [
    { "title": "Checkout" },
    { "title": "Refunds" }
  ]
}
```

## Markdown outline

```md
# Payment

Q2 delivery scope.

## Checkout

### Payment method selection

### Coupon deduction

## Refunds

### Original route refund

### Partial refund
```

Mapping reminders:

- Headings become topics.
- Heading hierarchy becomes topic hierarchy.
- Paragraph text under a heading becomes the topic note.
- Lists are supported by the full Markdown outline contract.

Read `docs/concepts/markdown-outline.md` before relying on list mapping.

## Patch: assert then add

```yaml
- op: assert_exists
  node: path:/Q2

- op: add_tree
  parent: path:/Q2
  position: last
  if_exists: error
  tree:
    title: Payment
    children:
      - title: Checkout
      - title: Refunds
```

Run:

```bash
xmind patch plan.xmind --ops ops.yaml --dry-run --json
```

## Patch: update fields

```yaml
- op: set
  node: id:topic-123
  fields:
    title: Payments
    note: Q2 payment scope
```

Set a field to `null` to clear it:

```yaml
- op: set
  node: id:topic-123
  fields:
    note: null
    image: null
```

## Patch: merge generated branch

```yaml
- op: merge_tree
  target: id:topic-123
  match_by: title_path
  tree:
    title: Payment
    children:
      - title: Checkout
      - title: Refunds
```

Use `prune: true` only when the user wants unmatched existing descendants
removed.

## Patch: reorganize

```yaml
- op: ensure_path
  path: /Q3/Payment

- op: move
  node: id:topic-456
  to: path:/Q3/Payment
  position: last

- op: sort_children
  node: path:/Q3/Payment
  by: title
  order: asc
```

## Image topic input

```yaml
title: Architecture
image:
  path: ./architecture.png
  alt: Architecture diagram
```

If image handling fails, follow the structured error. For unsupported asset
types, convert or remove the asset before retrying.

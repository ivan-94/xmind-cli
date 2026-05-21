# Topic Tree Schema

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Documentation schema for tree input objects
- Last updated: 2026-05-21

## Shape

```yaml
id: string?
title: string
note: string?
labels: string[]?
markers: string[]?
hyperlink: string?
image: TopicImage?
children: TopicTree[]?
```

## Field Rules

| Field | Required | Type | Notes |
| --- | --- | --- | --- |
| `id` | no | string | Existing topic id for id-based merge inputs; ignored for normal creates |
| `title` | yes | string | Must not be empty |
| `note` | no | string | Plain text or Markdown-like text |
| `labels` | no | string[] | Free-form labels |
| `markers` | no | string[] | Marker ids or semantic marker names |
| `hyperlink` | no | string | URL or supported link target |
| `image` | no | object | Local image path or existing asset id |
| `children` | no | TopicTree[] | Ordered child topics |

## TopicImage

```yaml
path: string?
asset_id: string?
alt: string?
title: string?
```

Exactly one of `path` or `asset_id` is required when `image` is present.

## Example

```yaml
title: 支付能力
labels: [MVP]
image:
  path: ./payment.png
  alt: Payment capability diagram
children:
  - title: 收银台
  - title: 退款
```

## Validation

Invalid examples:

```yaml
title: ""
```

```yaml
title: Payment
children: Payment child
```

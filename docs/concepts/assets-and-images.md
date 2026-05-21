# Assets and Images

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Image, attachment, and workbook resource behavior for the CLI product contract
- Last updated: 2026-05-21

## Purpose

XMind workbooks may contain binary resources such as topic images, local attachments, thumbnails, icons, and other package assets. The CLI must make these resources discoverable and preservable without forcing agents to understand workbook storage internals.

## Product Position

The first product contract supports structural asset management, not visual image editing.

The CLI should:

- preserve existing assets and references by default,
- list workbook resources in stable JSON,
- expose topic-level image references,
- attach an existing local image file to a topic,
- remove an image reference from a topic when explicitly requested,
- export embedded assets to a directory,
- report unsupported asset types clearly.

The CLI should not:

- crop, resize, compress, recolor, or visually edit images,
- guarantee pixel-perfect layout after changing images,
- expose internal package paths as the primary API,
- silently drop unknown binary resources.

## Domain Model

```text
workbook
  resources
    asset
      id
      kind
      media_type
      file_name
      byte_size
      checksum
      used_by

topic
  image
    asset_id
    alt
    title
```

## Asset Kinds

| Kind | Read | Preserve | Query | Edit Reference | Create Asset | Delete Asset |
| --- | --- | --- | --- | --- | --- | --- |
| Topic image | yes | yes | yes | yes | yes | reference-only |
| Attachment | yes | yes | yes | planned | planned | planned |
| Thumbnail | yes | yes | no | no | no | no |
| Unknown resource | metadata-only | yes | no | no | no | no |

`reference-only` means the CLI can remove a topic's link to an asset, but should garbage-collect the binary asset only when an explicit cleanup command is later designed.

## Commands

Asset behavior appears in existing commands first:

```bash
xmind get plan.xmind --node "id:topic-123" --include-assets --json
xmind tree plan.xmind --fields id,path,title,image --json
xmind set plan.xmind --node "id:topic-123" --image ./diagram.png --image-alt "Architecture diagram" --apply
xmind set plan.xmind --node "id:topic-123" --clear image --apply
xmind export plan.xmind --format assets --output ./assets
```

Future dedicated commands may include:

```bash
xmind assets plan.xmind --json
xmind asset-export plan.xmind --output ./assets
```

Those commands are deferred until the implementation needs stronger resource workflows.

## JSON Shape

Topic image reference:

```json
{
  "image": {
    "asset_id": "asset-123",
    "media_type": "image/png",
    "file_name": "diagram.png",
    "byte_size": 18231,
    "alt": "Architecture diagram"
  }
}
```

Workbook asset:

```json
{
  "id": "asset-123",
  "kind": "topic_image",
  "media_type": "image/png",
  "file_name": "diagram.png",
  "byte_size": 18231,
  "checksum": "sha256:...",
  "used_by": [
    { "topic_id": "topic-123", "path": "/Roadmap/Q2/Architecture" }
  ]
}
```

## Error Behavior

Asset errors should include:

- asset path,
- media type when detected,
- topic selector,
- whether the original workbook was left unchanged,
- suggested next action.

Example:

```json
{
  "code": "unsupported_asset_type",
  "message": "Only PNG, JPEG, GIF, and SVG topic images are supported for attachment.",
  "path": "./diagram.tiff",
  "retryable": false,
  "suggested_fix": "Convert the image to PNG or JPEG and retry."
}
```


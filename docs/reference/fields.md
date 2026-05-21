# Fields

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Legal field names for `--fields` and compact JSON payloads
- Last updated: 2026-05-21

## Purpose

`--fields` limits JSON payloads for agent context control. It does not change workbook data.

## Topic Fields

| Field | Meaning |
| --- | --- |
| `id` | Topic id |
| `path` | Canonical path relative to selected sheet root |
| `title` | Topic title |
| `note` | Topic note text |
| `labels` | Labels |
| `markers` | Markers |
| `hyperlink` | Hyperlink target |
| `image` | Topic image reference |
| `depth` | Depth under selected root |
| `children_count` | Direct child count |
| `children` | Nested child topics |

## Sheet Fields

| Field | Meaning |
| --- | --- |
| `id` | Sheet id |
| `index` | Zero-based sheet order |
| `title` | Sheet title |
| `root_topic_id` | Root topic id |
| `topic_count` | Topic count |

## Workbook Fields

| Field | Meaning |
| --- | --- |
| `file` | Workbook path |
| `format` | Detected XMind format |
| `sheet_count` | Number of sheets |
| `sheets` | Sheet summaries |
| `resources_count` | Number of resources |
| `capabilities` | Capability summary |

## Command Defaults

| Command | Default fields |
| --- | --- |
| `tree` | `id,path,title,children_count,children` |
| `get` | `id,path,title,note,labels,markers,hyperlink,image,children_count` |
| `find` | `id,path,title,sheet,children_count` |
| `sheets` | `id,index,title,topic_count` |
| `inspect` | `file,format,sheet_count,sheets,capabilities` |

## Syntax

```bash
xmind tree roadmap.xmind --json --format compact-json --fields id,path,title --depth 2
```

Unknown fields fail with `invalid_usage` and include `field_path: "fields"`.

Read command pages reference this list instead of redefining legal field names.

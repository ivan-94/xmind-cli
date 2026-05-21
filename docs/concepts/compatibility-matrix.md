# XMind Compatibility Matrix

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Compatibility contract for XMind features and editing capabilities
- Last updated: 2026-05-21

## Purpose

This matrix defines what the CLI promises to read, preserve, query, edit, create, and delete. It separates supported behavior from implementation internals so agents do not infer capabilities from examples alone.

## Capability Levels

| Level | Meaning |
| --- | --- |
| `read` | Returned by read commands when requested |
| `preserve` | Kept intact across unrelated writes |
| `query` | Usable in `find` or `query:` selectors |
| `edit` | Existing value can be changed |
| `create` | New value can be created |
| `delete` | Existing value or reference can be removed |
| `deferred` | Intentionally not in the first product contract |

## Feature Matrix

| XMind Feature | Read | Preserve | Query | Edit | Create | Delete | Notes |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Workbook | yes | yes | no | no | no | no | File-level object |
| Sheets | yes | yes | title/id/index | deferred | deferred | deferred | Initial contract reads existing sheets |
| Root topic | yes | yes | yes | limited | no | no | Root title can be edited only if XMind format permits |
| Child topics | yes | yes | yes | yes | yes | yes | Core supported object |
| Topic title | yes | yes | yes | yes | yes | yes | Core field |
| Topic note | yes | yes | contains/exact | yes | yes | yes | Plain text contract first |
| Labels | yes | yes | yes | yes | yes | yes | Explicit add/remove/set/clear semantics |
| Markers | yes | yes | yes | yes | yes | yes | Marker catalog validation may be format-dependent |
| Hyperlinks | yes | yes | yes | yes | yes | yes | URL or supported XMind link target |
| Topic images | yes | yes | by presence/type | reference | yes | reference | See `assets-and-images.md` |
| Attachments | metadata | yes | by presence/type | deferred | deferred | deferred | Preserve by default |
| Relationships | read-lite | yes | deferred | deferred | deferred | deferred | Preserve first, edit later |
| Summaries | read-lite | yes | deferred | deferred | deferred | deferred | Preserve first, edit later |
| Boundaries | read-lite | yes | deferred | deferred | deferred | deferred | Preserve first, edit later |
| Styles | metadata | yes | deferred | deferred | deferred | deferred | No style mutation initially |
| Themes | metadata | yes | deferred | deferred | deferred | deferred | No theme mutation initially |
| Stickers/icons | metadata | yes | deferred | deferred | deferred | deferred | Distinct from markers when format exposes them |
| Comments | metadata | yes | deferred | deferred | deferred | deferred | Preserve if present |
| Task info | metadata | yes | deferred | deferred | deferred | deferred | Preserve if present |
| Unknown fields | metadata | yes | no | no | no | no | Must not be dropped |

## Supported Editing Surface

The initial editing surface is:

- topic title,
- topic note,
- labels,
- markers,
- hyperlink,
- ordered child topics,
- subtree insertion,
- subtree replacement,
- subtree merge,
- subtree deletion,
- subtree move/copy,
- topic image reference and image asset creation.

Everything else is either read-lite or preserve-only unless a later reference page promotes it.

## Read-Lite Meaning

`read-lite` means the CLI may report that the feature exists and may include ids or endpoints needed for preservation, but it does not promise full semantic editing of that feature.

## Agent Rule

Agents should treat this matrix as authoritative. If a feature is `preserve` but not `edit`, the agent may rely on it surviving unrelated edits but should not attempt to change it.


# Domain Model

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Shared domain model used by every command
- Last updated: 2026-05-21

## Overview

The CLI exposes XMind files through a semantic model:

```text
workbook
  sheet
    topic tree
      topic
        title
        note
        labels
        markers
        hyperlink
        image
        children
        relationships
        summaries
        boundaries
```

The model intentionally hides the package format and internal storage layout.

## Workbook

A workbook is the whole `.xmind` file. It contains one or more sheets and shared resources.

Important workbook fields:

- `file`: input file path.
- `format`: detected XMind format.
- `sheets`: ordered sheet list.
- `resources`: attachments, thumbnails, and other preserved package data.

## Sheet

A sheet is a top-level map page inside the workbook.

Important sheet fields:

- `id`: stable sheet identifier when available.
- `title`: user-visible sheet title.
- `index`: zero-based sheet order.
- `root_topic`: root topic for the sheet.

Commands that operate on topics should accept `--sheet` when the workbook has multiple sheets.

## Topic

A topic is the primary editable node.

Important topic fields:

- `id`: topic identifier.
- `title`: visible topic text.
- `path`: CLI-computed topic path.
- `note`: longer body text.
- `labels`: short free-form tags.
- `markers`: marker ids or semantic marker names.
- `hyperlink`: optional link target.
- `image`: optional topic image reference.
- `children`: ordered child topics.

## Topic Path

A path is a slash-delimited address from a sheet root:

```text
/Roadmap/Q2/Payment/Refunds
```

Paths are human-friendly and useful for agents, but ids remain the most stable selector if a topic is renamed.

## Relationships, Summaries, and Boundaries

These features may be supported incrementally. The first product contract should preserve them even when not editing them. Future commands can expose them through dedicated reference pages instead of mixing every visual concept into topic commands.

## Assets and Compatibility

Topic image and workbook resource behavior is defined in `assets-and-images.md`. Full read, preserve, query, edit, create, and delete support is defined in `compatibility-matrix.md`.

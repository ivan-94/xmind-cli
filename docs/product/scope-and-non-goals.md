# Scope and Non-Goals

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Product scope boundaries for the XMind CLI
- Last updated: 2026-05-21

## In Scope

- Reading `.xmind` workbooks.
- Listing sheets.
- Rendering topic trees as outline, JSON, Markdown, or plain text.
- Searching topics by title, path, notes, labels, markers, hyperlink, and metadata.
- Editing topics and topic metadata.
- Attaching, listing, preserving, and exporting topic images at the reference/resource level.
- Adding, replacing, merging, deleting, moving, and copying subtrees.
- Applying declarative patch files.
- Producing diffs for planned or applied changes.
- Creating backups and restoring from backups.
- Validating workbook structure after changes.
- Preserving unknown XMind fields by default.
- Exporting and importing outline-friendly representations.

## Out of Scope for the First Product Definition

- Designing a graphical XMind editor.
- Reimplementing every visual layout feature of XMind.
- Guaranteeing pixel-perfect rendering.
- Synchronizing with cloud storage providers.
- Real-time collaborative editing.
- Visually editing binary images or attachments, such as resizing, cropping, compressing, or recoloring.
- Editing arbitrary binary attachments beyond preserving and referencing them.
- Exposing internal storage paths as the primary user model.

## Deferred but Plausible

- Relationship editing beyond read-lite preservation.
- Boundary and summary editing beyond read-lite preservation.
- Marker catalog introspection.
- Theme and style manipulation.
- Conflict-aware collaborative merge.
- LSP-style server mode for editor integrations.
- Watch mode for automated validation after file changes.

## Product Boundary

The CLI should be excellent at structural editing. It does not need to replace XMind as a visual thinking tool. It should make XMind files programmable, reviewable, and safe for agent workflows.

Detailed feature support is defined in `docs/concepts/compatibility-matrix.md`.

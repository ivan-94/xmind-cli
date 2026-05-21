# XMind Storage

## Source Manifest

- Conversation: XMind CLI product and technical design discussion
- Scope: XMind package reader/writer and preservation design
- Last updated: 2026-05-21

## Storage Goals

The storage layer must:

- read supported `.xmind` packages,
- detect unsupported package variants clearly,
- preserve unknown package entries,
- preserve unknown JSON fields,
- write valid packages after supported edits,
- keep binary assets intact unless explicitly changed.

## Package Reader

The reader should:

1. Open the workbook as a zip archive.
2. Locate known XMind content entries.
3. Decode known JSON structures into storage DTOs.
4. Convert storage DTOs into domain model.
5. Store unknown fields and package entries in preservation structures.

The domain layer should not know zip entry names.

## Package Writer

The writer should:

1. Convert the edited domain model back into storage DTOs.
2. Merge preserved unknown JSON fields.
3. Reuse preserved package entries.
4. Add or update asset entries when topic images are attached.
5. Write to a temporary file.
6. Validate the temporary workbook.
7. Atomically replace the destination.

## Format Support Strategy

First implementation target:

- modern XMind JSON-based `.xmind` packages,
- multiple sheets,
- topic tree,
- title,
- note,
- labels,
- markers,
- hyperlink,
- topic image references where the format exposes them cleanly.

Unsupported format variants should return:

```json
{
  "code": "unsupported_format",
  "retryable": false,
  "suggested_fix": "Open and re-save the file with a supported XMind version, or use export/import."
}
```

## Asset Handling

Assets should be tracked through a `ResourceIndex`:

```rust
pub struct ResourceIndex {
    pub assets: BTreeMap<AssetId, Asset>,
}
```

Adding an image should:

- verify media type,
- compute checksum,
- add the binary to the package,
- create or update topic image reference,
- preserve unrelated assets.

Removing an image should remove the topic reference. Binary garbage collection should be deferred until an explicit cleanup command exists.

## Filesystem Transactions

Workbook writes should use:

```text
destination.xmind
temporary file in destination directory
optional backup file
atomic rename
```

Writing the temporary file in the same directory improves atomic rename behavior across filesystems.

## Validation

Validation should run against the temporary output package, not the original package. If validation fails, the destination file remains untouched.


# xmind-cli Package Debugging

Use this reference for real `.xmind` files that fail to parse, validate, or
round-trip. The default posture is read-only package inspection.

## First prove CLI behavior

Capture the CLI result before inspecting internals:

```bash
xmind inspect real.xmind --json
xmind validate real.xmind --json
xmind tree real.xmind --depth 2 --json
```

If a command fails, preserve the JSON error code, message, suggested fix, and
any diagnostics. Do not mask the CLI error with a manual zip diagnosis.

## Read-only zip inspection

`.xmind` files are zip packages. Inspect entries without modifying the archive:

```bash
unzip -l real.xmind
unzip -t real.xmind
```

For modern JSON-based workbooks, look for known JSON content entries such as
`content.json`, metadata entries, manifest entries, and asset folders. The
storage layer is responsible for mapping package entry names into the domain
model; the domain model should not depend on zip names.

## JSON inspection

Extract JSON to a temporary directory:

```bash
tmp=$(mktemp -d)
unzip -q real.xmind -d "$tmp"
find "$tmp" -maxdepth 3 -type f | sort
python3 -m json.tool "$tmp/content.json" >/tmp/content.pretty.json
```

Keep this read-only. If `content.json` is missing or malformed, report that
alongside the CLI error.

## Preservation expectations

The CLI storage layer is expected to:

- preserve unknown package entries,
- preserve unknown JSON fields,
- keep unrelated binary assets intact,
- add or update assets only when topic images are changed,
- validate temporary output before replacing the destination.

If a real workbook exposes a preservation risk, create a minimal fixture or
record the exact package feature before changing code.

## Unsupported format variants

Unsupported packages should surface an explicit structured error, not a generic
panic or partial write. The expected recovery message is to open and re-save the
file with a supported XMind version, or use export/import when available.

## Low-level repair boundary

Only modify zip contents directly when the user explicitly asks for repair and
accepts the risk. Even then:

1. Copy the original file first.
2. Work in a temporary directory.
3. Repack to a new `.xmind` path.
4. Run `xmind validate repaired.xmind --json`.
5. Keep the original workbook unchanged.

Prefer CLI mutations for normal content edits.

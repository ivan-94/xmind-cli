# Workbook, Sheet, and Topic

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Practical explanation of workbook, sheet, and topic addressing
- Last updated: 2026-05-21

## Workbook

Most commands take the workbook path as their first positional argument:

```bash
xmind tree roadmap.xmind
```

Commands should not mutate the workbook unless they are write commands and `--apply` is present. `--dry-run` must never write the workbook.

## Sheet Selection

If a workbook has one sheet, topic commands may omit `--sheet`.

If a workbook has multiple sheets and a selector could match multiple sheets, the command fails unless the user supplies one of:

```bash
--sheet "Roadmap"
--sheet-id "sheet-abc"
--sheet-index 0
```

## Topic Selection

Topic commands use selectors:

```bash
xmind get roadmap.xmind --node "path:/Q2/Payment"
xmind set roadmap.xmind --node "id:topic-123" --title "Payments"
```

## Root Topics

Every sheet has a root topic. The root topic can be addressed by:

```text
root
path:/
```

`path:/` is the only canonical path selector for the selected sheet root. Canonical paths do not include the root topic title. The root title is returned as topic data, not as a path segment.

## Ordering

Child topics are ordered. Commands that add or move topics support position options:

```text
first
last
index:<zero-based-index>
before:<selector>
after:<selector>
```

Default insertion position is `last`.

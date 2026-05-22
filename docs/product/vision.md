# Product Vision

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Product-level vision for an agent-friendly XMind CLI
- Last updated: 2026-05-21

## Vision

`xmind` is a command line interface that lets agents and humans manipulate XMind files through a stable, semantic model. It hides storage details and exposes the mind map as an editable tree with sheets, topics, metadata, relationships, and summaries.

The primary product promise is:

> An agent can safely inspect, locate, edit, refactor, and validate an XMind file without understanding the XMind package format.

## Target Users

- Coding agents that need to maintain planning maps, product maps, architecture maps, or task trees.
- Human developers who want reliable scripted edits.
- Product managers who want outline-style imports and exports.
- Reviewers who need diffs and validation before accepting map changes.

## Core Jobs

- Inspect the structure of an `.xmind` file.
- Locate nodes by path, id, title, query, or metadata.
- Add, update, delete, move, copy, and replace topics.
- Insert or merge whole subtrees.
- Apply a declarative patch containing many operations.
- Export readable outlines for review.
- Validate that the file remains structurally safe.
- Produce stable machine-readable results for the next agent turn.

## Product Shape

The CLI should feel like a mix of:

- `jq`: structured input and stable JSON output.
- `git`: safe mutations, diffs, backups, and explicit application.
- `kubectl`: selector-based operations over domain objects.
- `tree`: fast visual inspection of hierarchy.

## North Star

An agent should be able to run this workflow without brittle assumptions:

```bash
xmind find roadmap.xmind --title "支付" --json
xmind get roadmap.xmind --node "id:topic-123" --depth 3 --json
xmind patch roadmap.xmind --ops payment.yaml --dry-run --json
xmind patch roadmap.xmind --ops payment.yaml --apply --backup --json
xmind validate roadmap.xmind --json
```

The result should be deterministic enough that a later agent can continue from the JSON outputs and documented selectors.

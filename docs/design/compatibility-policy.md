# Compatibility Policy

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Compatibility expectations for CLI behavior and documented schemas
- Last updated: 2026-05-21

## Stable Contracts

The following are stable once released:

- command names,
- documented options,
- JSON output fields,
- error codes,
- patch operation names,
- schema field meanings.

## Evolvable Areas

The following may evolve with less friction:

- human-readable text formatting,
- additional JSON fields,
- new command options,
- new patch operations,
- support for more XMind visual features.

## Breaking Changes

Breaking changes require a major version and migration notes:

- removing a command,
- changing a documented JSON field meaning,
- changing selector resolution semantics,
- changing default safety behavior,
- changing patch operation semantics.

## Agent Guidance

Agents should use `--json` and documented schemas. They should not parse default human output.


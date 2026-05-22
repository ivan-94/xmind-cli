# xmind completion

## Source Manifest

- Conversation: XMind CLI product design discussion
- Scope: Command reference for shell completion generation
- Last updated: 2026-05-22

## Purpose

Generate a shell completion script for the `xmind` CLI.

## Synopsis

```bash
xmind completion <shell>
```

## Arguments

- `<shell>`: target shell. Supported values are `bash`, `elvish`, `fish`, `powershell`, and `zsh`.

## Examples

```bash
xmind completion bash > xmind.bash
xmind completion zsh > _xmind
```

## Output

The command writes the generated completion script to stdout.

## Notes for Agents

This command does not read or write a workbook and does not use JSON envelopes.

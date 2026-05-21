# Triage Labels

The skills speak in terms of five canonical triage roles. This file maps those roles to the actual label strings used in this repo's issue tracker.

| Label in mattpocock/skills | Label in our tracker | Meaning                                  |
| -------------------------- | -------------------- | ---------------------------------------- |
| `needs-triage`             | `needs-triage`       | Maintainer needs to evaluate this issue  |
| `needs-info`               | `needs-info`         | Waiting on reporter for more information |
| `ready-for-agent`          | `ready-for-agent`    | Fully specified, ready for an AFK agent  |
| `ready-for-human`          | `ready-for-human`    | Requires human implementation            |
| `wontfix`                  | `wontfix`            | Will not be actioned                     |

When a skill mentions a role (e.g. "apply the AFK-ready triage label"), use the corresponding label string from this table.

Edit the right-hand column to match whatever vocabulary you actually use.

## Source Manifest

### Sources

- User confirmation in this setup session: keep the default five-role vocabulary.
- Skill template: `/Users/ivan/.agents/skills/setup-matt-pocock-skills/triage-labels.md`.

### Produced artifacts

- `docs/agents/triage-labels.md`
- `AGENTS.md`

### Key decisions

- Use default label strings for all five canonical triage roles.

### Verification evidence

- No existing repo-specific triage label configuration was found under `docs/agents/`.

### Open questions / risks

- If a future external tracker has existing labels with different names, update the right-hand column.

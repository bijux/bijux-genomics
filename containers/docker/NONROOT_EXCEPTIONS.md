# Docker Non-Root Exceptions

Purpose: Document tool images that currently run as root and why.

Policy:
- Preferred: set an explicit non-root `USER` in the final stage.
- Exception path: if non-root is not yet supported, tool ID must be listed here with rationale.

## Exceptions

| tool_id | rationale | owner |
|---|---|---|
| `*` | Transition period while legacy images are migrated to non-root runtime and write-path contracts. | `bijux-dna-platform` |

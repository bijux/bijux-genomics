# Docker Entrypoint/CMD Exceptions

Purpose: Track images that are temporarily exempt from strict ENTRYPOINT/CMD normalization.

Policy:
- Preferred contract: JSON `ENTRYPOINT` to tool binary and JSON `CMD` defaulting to `--help` (or `--version`).
- Exception path: tool ID listed here with rationale.

## Exceptions

| tool_id | rationale | owner |
|---|---|---|
| `*` | Transition period while legacy Dockerfiles are migrated away from shell-wrapper entrypoints and non-standard CMD defaults. | `bijux-dna-platform` |

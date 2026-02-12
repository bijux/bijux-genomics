# BIJUX CLI Conventions

Canonical user-facing command structure across the ecosystem is:

`bijux <app> <command> [args...]`

Rules:

- `bijux` is the only root binary entrypoint.
- Application commands must be namespaced under `<app>` (for this workspace: `bijux dna ...`).
- Legacy aliases (for example `bijux-dna`) are temporary compatibility shims only and must emit deprecation warnings.
- Make/CI/docs should invoke `bijux dna ...` and not app commands at root level.
- Global output flags (for example `--json`) are accepted at the root and apply to app subcommands.

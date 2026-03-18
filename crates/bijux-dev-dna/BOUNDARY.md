# bijux-dev-dna Boundary Contract

## Why this crate exists
Provides versioned control-plane automation for repository checks, domain automation, and other developer workflows.

## Allowed dependencies
- Workspace crates required to model and execute developer automation.
- External dependencies limited to CLI, parsing, and filesystem/process orchestration support.
- No reverse coupling from production runtime crates into this crate.

## Allowed effects
- Controlled filesystem reads and writes inside the workspace.
- Controlled process execution for delegated policy tests and automation entrypoints.
- No implicit network access unless a specific automation command owns that responsibility.

## Notes
This crate owns the durable registry and execution surface for repository checks, domain automation, and the migrated developer control-plane commands.

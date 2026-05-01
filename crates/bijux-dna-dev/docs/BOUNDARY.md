# bijux-dna-dev Boundary Contract

Owner: Developer automation
Scope: Repository control-plane automation, command catalogs, checks, and developer workflows
Allowed inputs: workspace files, command catalogs, explicit CLI arguments, delegated check results
Forbidden dependencies: production runtime ownership, domain semantics ownership, hidden workflow execution
Forbidden effects: undeclared network access, writes outside workspace-governed paths, product pipeline execution
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-dev --no-default-features`

## Why this crate exists
Provides versioned control-plane automation for repository checks, domain automation, container governance, and other developer workflows.

## Internal architecture
- `cli`: command-line parsing and user-facing control-plane entrypoint.
- `application`: group-level orchestration over catalogs, commands, and runtime context.
- `catalog`: versioned command and check catalogs.
- `commands`: native command execution and repository contract enforcement.
- `model`: typed command, check, and outcome definitions.
- `runtime`: workspace, filesystem, and process adapters.

## Allowed dependencies
- Workspace crates required to model and execute developer automation.
- External dependencies limited to CLI, parsing, and filesystem/process orchestration support.
- No reverse coupling from production runtime crates into this crate.

## Allowed effects
- Controlled filesystem reads and writes inside the workspace.
- Controlled process execution for delegated policy tests and automation entrypoints.
- No implicit network access unless a specific automation command owns that responsibility.

## Notes
This crate owns the durable command catalog and execution surface for repository automation.
The repository-level onboarding policy lives at `README.md`; child-repository work must apply that policy before changing this crate.

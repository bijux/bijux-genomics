# Architecture

`bijux-dna` is the CLI boundary for the genomics workspace. It owns argument
parsing, command routing, process context, output rendering, help snapshots, and
operator-facing exit behavior.

## Source Map

- `src/bin/bijux-dna.rs` is the binary wrapper.
- `src/cli_entrypoint.rs` exposes the testable CLI entrypoint.
- `src/process_exit.rs` maps categorized failures to process exits.
- `src/commands/` owns command parsing, routing, rendering, and CLI-only
  orchestration.
- `src/public_api/` exposes the small crate-local surface used by tests and
  downstream command documentation.

## Test Map

- `tests/boundaries.rs` checks source layout, dependency boundaries, command
  inventory, and help/documentation contracts.
- `tests/contracts.rs` checks CLI behavior and dry-run output contracts.
- `tests/guardrails.rs` runs shared policy guardrails for the crate.
- `tests/workspace_paths.rs` is intentionally present for workspace-path
  contract coverage when tests are selected by name.
- `tests/contracts/` and `tests/snapshots/` hold focused contract fixtures and
  rendered help snapshots.

## Boundaries

The CLI must not own science semantics, planner policy, engine internals,
runner backends, or hidden filesystem effects. It may coordinate declared CLI
commands and delegate durable behavior to API, domain, infrastructure, and
runtime crates where the dependency contract allows it.

## Dependency Direction

`bijux-dna` is an edge adapter. It may depend on the public API, selected
domain/compiler/runtime helpers required by commands, and infrastructure helpers
for declared effects. Lower-level crates must not depend on the CLI.

## Command Inventory

`docs/COMMANDS.md` is the SSOT for commands managed by this crate. Update it
with command routing, help snapshots, and command contract tests in the same
change.

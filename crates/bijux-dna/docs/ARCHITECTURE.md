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
- `tests/support/workspace_paths.rs` contains workspace-root helpers shared by
  integration suites without making support code look like a standalone suite.
- `tests/support/fixtures/process.rs` owns shared fixture-command process context
  and JSON output decoding for the BAM and ancient-DNA integration suites.
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

Benchmark taxonomy judgment separates declared corpus truth from classifier summary observation. `bijux-dna-bench-model` owns typed output records, the supported summary codec and pure expected-presence/count evaluation. `bijux-dna-bench` acquires explicit manifest/table/report files and publishes the existing JSON result. The CLI requests those services through the API benchmark facade. Summary aliases and legacy verdicts remain compatible; summary admission does not validate the full classifier stage report or establish biological truth.

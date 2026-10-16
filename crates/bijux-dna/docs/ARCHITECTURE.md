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
- `tests/bench_readiness_all_domain_completion.rs` groups parser observations,
  completion rules, active-scope and operational completion gates while preserving
  each public CLI report contract and its isolated runtime roots.
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

Benchmark stage-tool assets separate explicit planner/corpus acquisition and TOML publication from required binding policy. `readiness/stage_tool_assets.rs` delegates governed FASTQ/BAM asset coverage to the existing benchmark-model contract through the API facade; domain planning, path canonicalization, public command output and filesystem effects remain in their existing adapters.

Stage-tool resource rendering acquires execution specifications and tiny BAM fixture sizes, composes existing planner/domain resource constraints, and publishes the governed TOML/report. The pure benchmark walltime profiles are supplied by `bijux-dna-bench-model`; the CLI does not choose those stage/tool budgets.

Tool alias reports acquire their existing serving-map or candidate-matrix scope and delegate normalized cluster membership to the benchmark alias owner. CLI projection retains domain lists, explanations and the existing JSON/TSV formats. The benchmark alias owner classifies stage/tool violations across the acquired four readiness surfaces. The CLI retains acquisition order, report-envelope validation and publication before the existing stale-alias refusal.

The all-domain retained-tool adapter acquires executable bindings and external
artifact contracts, then delegates retained-row aggregation and source-closure
admission to the benchmark readiness owner. CLI report counts, model validation,
TSV presentation, output paths and publication remain at the command boundary.

Fixture-root validation keeps one successful FASTQ manifest observation within the current request in `commands/fixtures/root_validation/manifest_observation.rs`. The generic manifest row and named truth row retain their separate schemas and validity projections. Failed or differently selected validators remain independent; no observation survives the request or replaces stage materialization with a global cache.

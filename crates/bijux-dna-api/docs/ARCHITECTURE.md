# Architecture

`bijux-dna-api` is a boundary crate. It presents the stable v1 API while keeping
runtime wiring, support helpers, and domain-specific adapters private.

## Root Layout

- `Cargo.toml` defines the dependency graph and feature gates.
- `README.md` is the only root documentation file.
- `docs/` contains the 10 authoritative crate docs.
- `src/` contains the library implementation.
- `tests/` contains boundary, schema, contract, guardrail, and workspace-path
  integration tests.

## Source Layout

- `src/lib.rs` exposes `pub mod v1`; every other module is crate-private.
- `src/v1/api/front_door.rs` is the curated public front door.
- `src/v1/run/` owns public run entrypoints, request contracts, runtime support
  exports, and operator failure contracts.
- `src/v1/report/` owns report request contracts, analysis exports, and HTML
  bundle rendering.
- `src/v1/bam/stage_planning/` isolates BAM stage argument planning from the
  public BAM namespace.
- `src/v1/bench/`, `src/v1/env/`, `src/v1/fastq/`, and `src/v1/pipelines/`
  expose narrow v1 helper namespaces.
- `src/surface/` owns stable request/response contracts and explainability
  contracts used by the public API.
- `src/runtime/` adapts public requests into execution, validation, persistence,
  invocation-policy, and run/reporting behavior.
- `src/runtime/run/planning/` separates profile selection, run bootstrap, and
  planning support from execution.
- `src/runtime/run/execution/` contains the explicit stage-execution entrypoint.
- `src/runtime/run/reporting/` owns dry-run, execute, status, replay, report
  rendering, plan response materialization, summary artifacts, and workspace
  audit output.
- `src/support/` contains API-local support for benchmark runtime selection,
  QA gates, reference resolution, tool selection, and workspace registry/root
  resolution.
- `src/internal/` contains private cross-domain and FASTQ handler wiring.

## Test Layout

- `tests/boundaries/` protects architecture, docs layout, and API guardrails.
- `tests/schemas/` protects public schema snapshots and public surface exports.
- `tests/contracts/` exercises v1 behavior across public contract flows.
- `tests/snapshots/` stores governed insta snapshots for response contracts.
- `tests/workspace_paths.rs` protects workspace path behavior.

Test documentation lives in `docs/TESTS.md`; README files are intentionally not
allowed below `tests/`.

## Dependency Direction

The API crate may depend on lower-level planner, runner, runtime, environment,
domain, pipeline, stage-contract, analyzer, benchmark, and infrastructure crates
to compose v1 workflows. Lower-level crates must not depend on
`bijux-dna-api`.

The API crate must not bypass the runner/runtime boundary for process execution.
Stage adapters may prepare arguments, validate inputs, and call typed runner or
runtime APIs; they must not perform ad hoc shell orchestration.

## Change Rules

- Keep public exports behind `src/v1/api/front_door.rs`.
- Keep schema-bearing types in `src/surface/` or versioned `src/v1/` modules.
- Keep run execution separate from planning and reporting.
- Keep support helpers private unless they are deliberately exported through the
  v1 front door.
- Update `tests/boundaries/architecture.rs` and this document together when the
  source tree changes intentionally.

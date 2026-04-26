# bijux-dna-science Boundary Contract

Owner: Science control plane
Scope: Authored science evidence specs, deterministic traceability outputs, and release bundles
Allowed inputs: science specs, governed templates, deterministic fixtures, release metadata
Forbidden dependencies: runner backends, stage executors, planners, pipeline runtime crates
Forbidden effects: workflow execution, stage orchestration, direct tool launching, hidden container side effects
Validation command: `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-science --no-default-features`

## Why this crate exists
`bijux-dna-science` compiles authored science evidence specs into deterministic traceability
outputs and release bundles.

## Allowed dependencies
- Workspace core, policy-neutral parsing, rendering, and serialization crates required for the
  science control plane.
- No runner, stage executor, planner, or pipeline runtime dependency.

## Allowed effects
- Read authored specs from `science/specs/**`.
- Write governed generated science outputs under `science/generated/**`.
- Write immutable release bundles under `artifacts/science-releases/**`.

## Forbidden effects
- No workflow execution.
- No stage orchestration.
- No direct tool launching.
- No hidden benchmark, runtime, or container side effects.

## Notes
Boundary invariants are enforced by `bijux-dna-policies` contract tests.
The family-level contract is indexed in `docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md`.

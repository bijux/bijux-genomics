# bijux-dna-science Boundary Contract

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

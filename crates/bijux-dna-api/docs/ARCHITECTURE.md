# Architecture

## Layout
- `lib.rs` stays thin and exposes only the stable `v1` surface.
- `surface/` owns request/response contracts and explainability contracts used by the public API.
- `runtime/` owns execution/reporting adapters, runtime validation, persistence helpers, and invocation policy support.
- `runtime/invocation_policy/` isolates policy models, path contracts, and recovery artifacts from the top-level policy rules.
- `support/` owns workspace resolution, registry loading, tool eligibility, runner selection, and QA helpers.
- `internal/` owns non-public handler wiring, cross-domain adapters, and fastq-specific implementation details.
- `v1/` owns the curated public entrypoints and re-export policy for the stable API surface.

## Change rules
- Keep stable schema and explainability contracts under `surface/`, not at the crate root.
- Keep runtime adapters under `runtime/` and avoid routing through hidden crate-root shortcut aliases.
- Keep workspace registry loading separate from runner/tool policy in `support/`.
- Keep `internal/` wiring private and avoid exposing handler modules through the stable public surface.

## Data flow
- Public callers enter through `v1/`.
- `surface/` provides stable request and explainability contracts.
- `runtime/` plans, executes, dry-runs, reports, and audits through adapter modules.
- `support/` resolves repository-scoped inputs needed by the adapters.
- `internal/` handles domain-specific orchestration that is intentionally outside the public API contract.

# Architecture

## Layout
- `lib.rs` stays thin and exposes only the stable `v1` surface.
- `surface/` owns request/response contracts and explainability contracts used by the public API.
- `runtime/` owns execution/reporting adapters, runtime validation, persistence helpers, and invocation policy support.
- `runtime/invocation_policy/` isolates policy models, path contracts, and recovery artifacts from the top-level policy rules.
- `runtime/run/planning/` separates profile selection, run bootstrap, and planning support from the run facade.
- `runtime/run/execution/` isolates run execution entry from the surrounding run facade.
- `runtime/run/reporting/` isolates report rendering, dry-run/execute entrypoints, replay/status helpers, plan response materialization, summary artifacts, and workspace audit support from the run facade.
- `support/workspace/` owns repository root resolution and workspace registry loading.
- `support/tool_selection.rs` owns tool eligibility filtering.
- `support/benchmark_runtime.rs` owns benchmark runtime selection.
- `support/reference_resolution/` separates reference resolution contracts from the local filesystem adapter.
- `support/qa/` separates QA gate checks from the runner shim.
- `internal/` owns non-public handler wiring, cross-domain adapters, and fastq-specific implementation details.
- `internal/fastq/stage_ids/` separates fastq stage constants by source authority instead of flattening them into a vague root.
- `v1/` owns the curated public entrypoints and re-export policy for the stable API surface.
- `v1/api/` is the explicit public front door over the curated versioned namespaces.
- `v1/bench/`, `v1/env/`, `v1/fastq/`, and `v1/pipelines/` are explicit versioned namespaces instead of flat root files.
- `v1/run/` separates run entrypoints, request contracts, runtime support exports, and operator failure contracts.
- `v1/report/` separates report request contracts, analysis exports, and HTML bundle rendering.

## Change rules
- Keep stable schema and explainability contracts under `surface/`, not at the crate root.
- Keep runtime adapters under `runtime/` and avoid routing through hidden crate-root shortcut aliases.
- Keep workspace asset resolution under `support/workspace/`, not in flat support roots.
- Keep tool eligibility filtering separate from registry loading and benchmark runtime selection.
- Keep runtime report rendering and workspace audit support separate from run lifecycle orchestration.
- Keep `internal/` wiring private and avoid exposing handler modules through the stable public surface.

## Data flow
- Public callers enter through `v1/`.
- `surface/` provides stable request and explainability contracts.
- `runtime/` plans, executes, dry-runs, reports, and audits through adapter modules.
- `support/workspace/` resolves repository-scoped inputs needed by the adapters.
- `support/tool_selection.rs` and `support/benchmark_runtime.rs` enforce internal tool/runtime policy.
- `internal/` handles domain-specific orchestration that is intentionally outside the public API contract.

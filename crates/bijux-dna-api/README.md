# bijux-dna-api

## What this crate does
Public API surface for orchestration endpoints and schemas.

## What it must not do (boundaries)
No direct engine or runner calls. The API orchestrates planners and reads runtime artifacts only.

## Effects & determinism guarantees
Deterministic request handling for the same inputs; no side effects beyond HTTP I/O.

## Public API / entrypoints
See the stable v1 endpoints below and `crates/bijux-dna-api/docs/API.md`.

## Stable v1 endpoints
- `plan` → returns `PlanResponse`
- `execute` → returns `ExecuteResponse`
- `dry-run` → returns `DryRunResponse`
- `status` → returns `RunStatus`
- `explain` → returns `ExplainResponse`
- `policy-audit` → returns policy audit JSON

## Versioning rules
- The v1 API is the only stable surface.
- Schema changes require snapshot updates and explicit review.
- Compatibility rules live in `crates/bijux-dna-api/docs/API_STABILITY.md`.

## Contract snapshots (source of truth)
- `tests/snapshots/bijux-dna-api__schemas__plan_response_schema.snap`
- `tests/snapshots/bijux-dna-api__schemas__execute_response_schema.snap`
- `tests/snapshots/bijux-dna-api__schemas__dry_run_response_schema.snap`
- `tests/snapshots/bijux-dna-api__schemas__status_schema.snap`
- `tests/snapshots/bijux-dna-api__schemas__explain_schema.snap`
- `tests/snapshots/bijux-dna-api__schemas__policy_audit_schema.snap`

## Key contracts it owns/consumes
- Owns the public API response schemas: `crates/bijux-dna-api/docs/API.md` and `crates/bijux-dna-api/docs/API_STABILITY.md`.
- Stability snapshots: `tests/snapshots/bijux-dna-api__schemas__*.snap`.

## Artifacts / Contracts
- Response schemas in `crates/bijux-dna-api/docs/API.md` and snapshot tests under `tests/snapshots/`.
- Request/response flow contract in `crates/bijux-dna-api/docs/REQUEST_FLOW.md`.

## Failure modes
Most failures surface as schema drift (snapshot diffs) or handler contract mismatches.

## Internal handlers (non-public)
`src/internal/*` is not public API and may change at any time. It is for wiring and adapters only.

## Request flow
See `crates/bijux-dna-api/docs/REQUEST_FLOW.md` for how requests map to planners, engine, and runtime artifacts.

## Docs entrypoints
See `crates/bijux-dna-api/docs/INDEX.md`, `crates/bijux-dna-api/docs/API.md`, `crates/bijux-dna-api/docs/API_STABILITY.md`, `crates/bijux-dna-api/docs/REQUEST_FLOW.md`, `crates/bijux-dna-api/docs/BOUNDARIES.md`, `crates/bijux-dna-api/docs/CHANGE_RULES.md`.

## How to run its tests
See `crates/bijux-dna-api/docs/TESTS.md`. Key tests: `tests/schemas/api_stability.rs`, `tests/schemas/schema_snapshots.rs`,
`tests/contracts/v1_fastq_contract.rs`, `tests/contracts/v1_bam_contract.rs`, `tests/contracts/v1_cross_contract.rs`.

## Where the docs live
Start at `crates/bijux-dna-api/docs/INDEX.md`, then follow the API and stability docs above.

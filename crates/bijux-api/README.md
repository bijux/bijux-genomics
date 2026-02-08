# bijux-api

## What this crate does
Public API surface for orchestration endpoints and schemas.

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
- Compatibility rules live in `docs/API_STABILITY.md`.

## Contract snapshots (source of truth)
- `tests/snapshots/v1_cross_api_stability__plan_response_schema.snap`
- `tests/snapshots/v1_cross_api_stability__execute_response_schema.snap`
- `tests/snapshots/v1_cross_api_stability__dry_run_response_schema.snap`
- `tests/snapshots/v1_cross_api_stability__status_schema.snap`
- `tests/snapshots/v1_cross_api_stability__explain_schema.snap`
- `tests/snapshots/v1_cross_api_stability__policy_audit_schema.snap`

## Internal handlers (non-public)
`src/internal/*` is not public API and may change at any time. It is for wiring and adapters only.

## Request flow
See `docs/REQUEST_FLOW.md` for how requests map to planners, engine, and runtime artifacts.

## Docs entrypoints
See `docs/INDEX.md`, `docs/API.md`, `docs/API_STABILITY.md`, `docs/REQUEST_FLOW.md`, `docs/BOUNDARIES.md`, `docs/CHANGE_RULES.md`.

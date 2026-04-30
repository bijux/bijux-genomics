# Request Flow

The API crate is a Rust front door, not an HTTP server. Callers enter through
`bijux_dna_api::v1::api`, provide typed requests, and receive typed responses or
JSON audit/report evidence.

## Plan

1. Caller builds `PlanRequest`.
2. `v1::api::plan` routes to `runtime/run/reporting/plan_response.rs`.
3. The API validates the execution graph and profile context, then materializes
   a governed workflow manifest and plan manifest.
4. The response returns `PlanResponse` with the graph, graph hash, manifest
   preview, workflow manifest, plan manifest, and optional semantic diff.

No stages execute in this flow.

## Dry Run

1. Caller builds `DryRunRequest` with graph, run directory, and profile id.
2. `v1::api::dry_run` routes to `runtime/run/reporting/dry_run.rs`.
3. The API writes deterministic graph and manifest artifacts under the declared
   run directory, including `plan_manifest.json`.
4. The response returns `DryRunResponse` paths to those artifacts.

Dry-run may write declared artifacts, but it must not execute stage tools.

## Execute

1. Caller builds `ExecuteRequest` with graph, runtime kind, and run directory.
2. `v1::api::execute` routes to `runtime/run/reporting/execute_run.rs`.
3. The API validates input and delegates execution through runtime/runner
   boundaries.
4. The response returns `ExecuteResponse` with run id, manifest path, and
   optional report path.

Execution is the only managed flow allowed to invoke runner/runtime execution.

## Execute And Report

1. Caller invokes `execute_and_report` with `ExecuteRequest`.
2. Execution runs through the execute path.
3. Report rendering runs through `runtime/run/reporting/rendering.rs`.
4. The response remains an `ExecuteResponse`; report artifacts are declared by
   the response paths and run manifest.

## Status

1. Caller asks for status for a run directory.
2. `v1::api::status` reads persisted manifest/report evidence.
3. The response returns `RunStatus`.

Status must not mutate the run.

## Operator Controls

1. Caller invokes `pause_run`, `resume_run`, or `cancel_run` with a run
   directory.
2. The API updates `run_control.json` through the reporting control surface.
3. The response returns `RunControlResponse` with the durable control artifact
   path and current audited state.

Control operations must remain auditable and must not bypass the governed
runner/runtime state machine.

## Operator Health

1. Caller invokes `operator_health` with a run directory.
2. The API infers the runtime backend from the persisted executor descriptor.
3. The API writes `operator_health.json` and returns
   `OperatorHealthResponse`.

Health checks may write health evidence, but they must not execute workflow
stages.

## Explain

1. Caller provides an execution graph and optional defaults ledger.
2. `v1::api::explain` calls the explainability surface.
3. The response returns `ExplainResponse` with selected tools, defaults diff, and
   stage contract hashes when available.

Explainability is deterministic for the same graph and defaults ledger.

## Policy Audit

1. Caller invokes `policy_audit`, `workspace_edges`, or `write_workspace_audit`.
2. `policy_audit` returns the owning dev/policy crates and verification commands; API runtime code does not execute policy guardrails.
3. The flow returns JSON or persisted audit artifacts.

Audit output is evidence for this crate boundary; it must not change source
state.

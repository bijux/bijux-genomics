<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-docs -->

# API_VERSIONING

## Purpose
Generated inventory linking stable v1 API routes to the governed workflow, plan, runtime, and evidence schemas they read or surface.

## Scope
Route-level schema read/write surfaces declared by the v1 API inventory.

## Non-goals
- Replacing endpoint behavior documentation or payload examples.

## Contracts
- Generated-only document; manual edits are forbidden.
- Route entries must be sourced from the governed API inventory.

- Inventory schema: `bijux.api_route_inventory.v1`
- API version: `v1`

| Route | Response Struct | Reads | Writes |
|---|---|---|---|
| `v1.plan` | `PlanResponse` | `workflow_manifest` | `workflow_manifest, plan_manifest` |
| `v1.dry_run` | `DryRunResponse` | `workflow_manifest, plan_manifest` | `run_backend, run_scheduling_decision, run_queue_state, run_lease, run_control, operator_health, run_state, artifact_inventory, evidence_bundle, evidence_verification` |
| `v1.execute` | `ExecuteResponse` | `workflow_manifest, plan_manifest` | `run_backend, run_scheduling_decision, run_queue_state, run_lease, run_control, operator_health, slurm_submission, run_state, run_failure, artifact_inventory, evidence_bundle, evidence_verification, report` |
| `v1.status` | `RunStatus` | `run_backend, run_scheduling_decision, run_queue_state, run_lease, run_control, operator_health, slurm_submission, run_state, run_failure, artifact_inventory, evidence_bundle, evidence_verification` | `-` |
| `v1.pause_run` | `RunControlResponse` | `run_control, run_queue_state` | `run_control` |
| `v1.resume_run` | `RunControlResponse` | `run_control, run_queue_state` | `run_control` |
| `v1.cancel_run` | `RunControlResponse` | `run_control, run_queue_state` | `run_control` |
| `v1.operator_health` | `OperatorHealthResponse` | `run_backend, run_scheduling_decision, run_state` | `operator_health` |

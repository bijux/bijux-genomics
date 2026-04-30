<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-docs -->

# API_VERSIONING

## Purpose
Generated inventory linking stable v1 API routes to the governed workflow, plan, runtime, and evidence schemas they read or surface.

- Inventory schema: `bijux.api_route_inventory.v1`
- API version: `v1`

| Route | Response Struct | Reads | Writes |
|---|---|---|---|
| `v1.plan` | `PlanResponse` | `workflow_manifest` | `workflow_manifest, plan_manifest` |
| `v1.dry_run` | `DryRunResponse` | `workflow_manifest, plan_manifest` | `run_state, artifact_inventory, evidence_bundle, evidence_verification` |
| `v1.execute` | `ExecuteResponse` | `workflow_manifest, plan_manifest` | `run_state, run_failure, artifact_inventory, evidence_bundle, evidence_verification, report` |
| `v1.status` | `RunStatus` | `run_state, run_failure, artifact_inventory, evidence_bundle, evidence_verification` | `-` |

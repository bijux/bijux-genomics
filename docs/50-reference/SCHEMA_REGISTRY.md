<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-docs -->

# SCHEMA_REGISTRY

## Purpose
Generated registry of governed workflow, plan, runtime, evidence, metric, report, and error compatibility surfaces.

## Scope
Lists authoritative compatibility surfaces, their semantic versions, and durable error code ownership.

## Non-goals
Does not replace crate-level API docs, implementation details, or migration playbooks.

## Contracts
This page is generated from governed registries in code and must be updated via `cargo run -p bijux-dna-dev -- tooling run generate-docs`.

## Schema Families
| Family | Schema | Semantic Version | Surface | Compatibility | Migration Rule | Owner |
|---|---|---|---|---|---|---|
| `workflow_manifest` | `bijux.workflow_manifest.v1` | `1.0.0` | `workflow` | `migratable` | `upgrade_with_governed_tooling` | `bijux-dna-core` |
| `plan_manifest` | `bijux.plan_manifest.v1` | `1.0.0` | `plan` | `migratable` | `upgrade_with_governed_tooling` | `bijux-dna-core` |
| `artifact_inventory` | `bijux.artifact_inventory.v1` | `1.0.0` | `artifact` | `migratable` | `support_n_and_n_minus_one` | `bijux-dna-runtime` |
| `evidence_bundle` | `bijux.evidence_bundle.v1` | `1.0.0` | `evidence` | `migratable` | `support_n_and_n_minus_one` | `bijux-dna-analyze` |
| `evidence_verification` | `bijux.evidence_verification.v1` | `1.0.0` | `evidence` | `additive` | `add_optional_fields_only` | `bijux-dna-analyze` |
| `evidence_comparison` | `bijux.evidence_comparison.v1` | `1.0.0` | `evidence` | `additive` | `add_optional_fields_only` | `bijux-dna-analyze` |
| `metrics_envelope` | `bijux.metrics_envelope.v2` | `2.0.0` | `metric` | `exact_match` | `refuse_unknown_versions` | `bijux-dna-runtime` |
| `report` | `bijux.report.v1` | `1.0.0` | `report` | `additive` | `add_optional_fields_only` | `bijux-dna-analyze` |
| `run_backend` | `bijux.run_backend.v1` | `1.0.0` | `run_state` | `additive` | `add_optional_fields_only` | `bijux-dna-runtime` |
| `run_scheduling_decision` | `bijux.run_scheduling_decision.v1` | `1.0.0` | `run_state` | `additive` | `add_optional_fields_only` | `bijux-dna-runtime` |
| `run_queue_state` | `bijux.run_queue_state.v1` | `1.0.0` | `run_state` | `additive` | `add_optional_fields_only` | `bijux-dna-runtime` |
| `run_lease` | `bijux.run_lease.v1` | `1.0.0` | `run_state` | `additive` | `add_optional_fields_only` | `bijux-dna-runtime` |
| `run_control` | `bijux.run_control.v1` | `1.0.0` | `run_state` | `additive` | `add_optional_fields_only` | `bijux-dna-runtime` |
| `operator_health` | `bijux.operator_health.v1` | `1.0.0` | `run_state` | `additive` | `add_optional_fields_only` | `bijux-dna-runtime` |
| `slurm_submission` | `bijux.slurm_submission.v1` | `1.0.0` | `run_state` | `additive` | `add_optional_fields_only` | `bijux-dna-runtime` |
| `run_state` | `bijux.run_state.v1` | `1.0.0` | `run_state` | `migratable` | `support_n_and_n_minus_one` | `bijux-dna-runtime` |
| `run_failure` | `bijux.run_failure.v1` | `1.0.0` | `run_state` | `additive` | `add_optional_fields_only` | `bijux-dna-runtime` |

## Durable Error Codes
| Error ID | Area | Wire Code | Owner | Remediation |
|---|---|---|---|---|
| `contract.execution_output_mismatch` | `contract` | `execution_output_mismatch` | `bijux-dna-core` | Refresh the stage contract outputs or the emitting stage so runtime outputs and governed artifact promises match exactly. |
| `scientific.invariant_violation` | `scientific` | `invariant_violation` | `bijux-dna-runtime` | Inspect the stage scientific contract, reference context, and invariant evidence before admitting the run as enforced. |
| `runtime.runner_execution_failed` | `runtime` | `runner_execution_failed` | `bijux-dna-api` | Inspect run_failure.json, tool invocation logs, and telemetry for the failing stage before retrying or replaying the run. |
| `infrastructure.io_error` | `infrastructure` | `io_error` | `bijux-dna-core` | Verify governed paths exist under the run layout and that the active runtime has permission to read and write them. |
| `api.invalid_request` | `api` | `invalid_request` | `bijux-dna-api` | Rebuild the request from the v1 contract surface and confirm the workflow, plan, and runtime schemas match the reviewed adapters. |
| `cache.cache_key_mismatch` | `cache` | `cache_key_mismatch` | `bijux-dna-core` | Regenerate the plan manifest and compare cache identity fields, reference assets, and policy surfaces before reusing cached artifacts. |

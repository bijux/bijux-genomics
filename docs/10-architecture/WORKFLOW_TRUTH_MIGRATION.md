# Workflow Truth Migration

Move workflow truth out of `bijux-dna-dev` and CLI helper modules.

## Authority

- Pipeline-profile JSON payloads now belong to `crates/bijux-dna-api/src/v1/plan.rs`.
- Plan support artifacts now belong to `crates/bijux-dna-api/src/v1/run/runtime_support.rs`.
- CLI and dev surfaces consume those APIs and render results; they do not define the sidecar schemas.

## What moved

- `plan explain-profile` and `plan validate-profile` now delegate their canonical JSON payloads to the API planning surface.
- `plan_artifact_manifest.json`, `decision_trace.json`, and `policy_snapshot.json` are materialized by the API runtime support layer instead of the CLI planning entrypoint.

## Verification

- `crates/bijux-dna/tests/contracts/cli_library_parity.rs` proves representative FASTQ, BAM, and VCF planning outputs match the library APIs.
- `crates/bijux-dna-policies/tests/contracts/tooling/governance_core/workflow_truth_boundary_policy.rs` prevents the plan-sidecar schema versions from being redefined in CLI or dev code.

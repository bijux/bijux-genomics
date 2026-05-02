# Workflow Truth Migration

Move workflow truth out of `bijux-dna-dev` and CLI helper modules.

## Authority

- Pipeline-profile JSON payloads now belong to `crates/bijux-dna-api/src/v1/plan.rs`.
- Plan support artifacts now belong to `crates/bijux-dna-api/src/v1/run/runtime_support.rs`.
- CLI and dev surfaces consume those APIs and render results; they do not define the sidecar schemas.

## What moved

- `plan explain-profile` and `plan validate-profile` now delegate their canonical JSON payloads to the API planning surface.
- Cross-domain profile payloads now carry governed workflow-template, sample-sheet, batch, fan-in/fan-out, failure-policy, evidence-summary, and parameter-lock contracts from the API planning surface instead of being assembled in CLI helpers.
- `plan_artifact_manifest.json`, `decision_trace.json`, and `policy_snapshot.json` are materialized by the API runtime support layer instead of the CLI planning entrypoint.

## Verification

- `crates/bijux-dna/tests/contracts/cli_library_parity.rs` proves representative FASTQ, BAM, VCF, and cross-domain planning outputs match the library APIs.
- `crates/bijux-dna-policies/tests/contracts/tooling/governance_core/workflow_truth_boundary_policy.rs` prevents the plan-sidecar schema versions from being redefined in CLI or dev code.

## Purpose
This document describes the governed intent and operator-facing meaning of this surface.

## Scope
The scope is limited to repository-owned behavior, contracts, and evidence paths for this topic.

## Non-goals
This document does not redefine source-of-truth schemas, code ownership boundaries, or release policy outside this surface.

## Contracts
Claims here are valid only when they remain consistent with governed configs, domain authorities, and policy checks.


# REFERENCE_GOVERNANCE

FASTQ stages that require references must resolve species/build authority via `bijux-dna-db-ref` and use lock-backed materialized references from `artifacts/reference_store/.../refs/{raw,normalized,derived}`.

## Rules
- Do not hardcode host filesystem reference paths.
- Reference acquisition is only allowed through `scripts/tooling/acquire-reference.sh`.
- Runtime writes `run_artifacts/reference_manifest.json` for reference-consuming stages.


## Purpose

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.


## Scope

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.


## Non-Goals

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.


## Contracts

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.

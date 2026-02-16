# REFERENCE_GOVERNANCE

BAM stages must use `bijux-dna-db-ref` authority for species/build mapping and contig policy checks before executing tool wrappers.

## Rules
- Reference and index inputs must come from lock-backed materialization (`refs/raw`, `refs/normalized`, `refs/derived`).
- Contig/build mismatches are refusal conditions.
- Runtime emits `run_artifacts/reference_manifest.json` including checksums and authority payload when species/build params are present.


## Purpose

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.


## Scope

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.


## Non-Goals

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.


## Contracts

Contract details are enforced by stage contracts, schema locks, and CI policy gates for this scope.

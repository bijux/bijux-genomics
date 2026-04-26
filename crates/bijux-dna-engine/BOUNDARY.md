# bijux-dna-engine Boundary Contract

## Why this crate exists
Owns execution orchestration for an already planned `ExecutionGraph`.

## Allowed dependencies
- `bijux-dna-core` for graph, step, artifact, and run-record contracts.
- `bijux-dna-runtime` for the `Runner` interface and run layout contracts.
- `bijux-dna-infra` for filesystem helpers needed to record and verify engine-owned artifacts.

## Allowed effects
- May write engine-owned execution records under step `run_artifacts/`.
- May inspect declared output paths and required run artifacts.
- Must not spawn tools, select containers, contact networks, or own domain semantics.

## Required separation
- Planning belongs outside this crate.
- Backend execution belongs in runner/runtime layers.
- FASTQ/BAM/VCF semantics belong in domain, planner, and stage crates.

## Notes
Boundary invariants are enforced by bijux-dna-policies and the engine boundary tests.

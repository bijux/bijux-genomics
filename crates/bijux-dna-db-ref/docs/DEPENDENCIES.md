# Dependencies

`bijux-dna-db-ref` is a read-only data-source resolver. Its dependency graph
must stay below planners, stages, runners, APIs, and CLIs.

## Normal Dependencies

- `anyhow`: resolver error context and validation failures.
- `bijux-dna-domain-vcf`: shared VCF `SpeciesContext`, `ContigSpec`, and
  coverage regime contracts.
- `serde`: config and public contract serialization/deserialization.
- `toml`: checked-in runtime and VCF catalog config parsing.

## Dev Dependencies

- `bijux-dna-policies`: crate-local guardrail smoke coverage.

## Forbidden Workspace Dependencies

This crate must not depend on `bijux-dna`, API, planner, engine, runner,
runtime, environment, stage, benchmark, data-download, or infrastructure crates.

## Verification

The dependency boundary is locked by `tests/boundaries/dependency_graph.rs`.

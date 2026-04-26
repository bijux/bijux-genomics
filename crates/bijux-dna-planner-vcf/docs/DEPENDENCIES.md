# Dependencies

`bijux-dna-planner-vcf` keeps its dependency graph focused on planning contracts and VCF domain vocabulary.

## Runtime Dependencies
- `anyhow` for planner refusal errors.
- `serde` and `serde_json` for stable plan and explain payload serialization.
- `toml` for repository-owned registry parsing.
- `sha2` for deterministic graph and plan identifiers.
- `bijux-dna-core` for plan policy, artifact, command spec, and execution graph contracts.
- `bijux-dna-domain-vcf` for VCF stage taxonomy, coverage regimes, invariants, and downstream transition validation.
- `bijux-dna-db-ref` for reference bundle, panel, and map catalog handoff types.
- `bijux-dna-stage-contract` for stage plan payloads.

## Dev Dependencies
- `bijux-dna-policies` for guardrail configuration tests.

## Forbidden Dependency Families
- Runtime, runner, engine, CLI, API, environment, and product execution crates.
- FASTQ and BAM planner/stage crates.
- Network clients and external database clients.
- Benchmark model crates unless a future VCF benchmark planning boundary is explicitly designed.

## Boundary Rule
If a dependency is needed only to execute a planned command, observe the environment, or parse runtime outputs, it belongs outside this crate.

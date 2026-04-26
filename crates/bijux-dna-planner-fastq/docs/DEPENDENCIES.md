# Dependencies

`bijux-dna-planner-fastq` depends on crates needed to assemble deterministic FASTQ plans and cross FASTQ-to-BAM graph views. It must not depend on runtime execution, CLI, API, database, or analysis application crates.

## Runtime Dependencies
- `anyhow` — fallible planning and graph validation errors.
- `serde` — planner-local serializable preprocess policy types.
- `serde_json` — params, explain details, and snapshot payloads.
- `bijux-dna-core` — typed IDs, tool specs, graph specs, command specs, artifacts, and policy types.
- `bijux-dna-stage-contract` — `StagePlanV1`, execution-step projection, plan reasons, and graph edges.
- `bijux-dna-domain-fastq` — FASTQ stage vocabulary, params, invariants, tool governance, and domain contracts.
- `bijux-dna-domain-bam` — BAM stage IDs for cross FASTQ-to-BAM catalog projection only.
- `bijux-dna-pipelines` — FASTQ profiles and shared stage identifiers.
- `bijux-dna-stages-fastq` — FASTQ stage spec builders and runtime interpretation metadata.
- `bijux-dna-infra` — repository-owned configuration and fixture path helpers.
- `tracing` — planning graph diagnostics only.
- `toml` — registry/configuration parsing used by tests and support helpers.

## Test-Only Dependencies
- `bijux-dna-policies` — shared guardrail checks.
- `bijux-dna-testkit` — snapshot and fixture helpers.
- `flate2` — compressed FASTQ fixture support.
- `insta` — snapshot contracts.

## Forbidden Dependency Direction
This crate must not depend on runner, engine, CLI, API, database, environment, science orchestration, or analysis crates. Those crates may consume planner output; the planner must not consume them.

## Review Rules
- Keep execution and output parsing downstream.
- Keep FASTQ vocabulary and governance upstream in domain/stage/pipeline crates.
- Keep cross BAM coupling limited to stage ID catalog projection.
- Prefer workspace dependency declarations when the dependency is already listed at the workspace root.

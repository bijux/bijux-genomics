# Dependencies

`bijux-dna-planner-bam` is a deterministic planner. Dependencies must support contract ingestion, tool selection, stage plan construction, graph assembly, and tests.

## Runtime Dependencies
- `anyhow` — fallible plan assembly and boundary errors.
- `bijux-dna-core` — typed IDs, plan policy, execution graph, command specs, artifacts, and tool specs.
- `bijux-dna-domain-bam` — BAM stage vocabulary, params, invariants, and domain contracts.
- `bijux-dna-stage-contract` — `StagePlanV1`, plan reasons, execution-step projection, and graph edges.
- `bijux-dna-stages-bam` — BAM stage specs and stage-adapter contract data.
- `bijux-dna-pipelines` — BAM pipeline profiles and default stage ordering.
- `bijux-dna-infra` — repository-owned configuration path resolution for tool registry reads.
- `serde_json` — params, explain details, and snapshot payloads.
- `tracing` — planning graph diagnostics only.
- `toml` — repository tool registry parsing.

## Test-Only Dependencies
- `bijux-dna-policies` — shared guardrail checks.
- `bijux-dna-testkit` — snapshot helpers and fixtures.
- `insta` — snapshot contracts.

## Forbidden Dependency Direction
This crate must not depend on runner, engine, CLI, API, database, environment, science orchestration, or analysis application crates. Those crates may consume planner output; the planner must not consume them.

## Review Rules
- Keep execution and runtime discovery downstream.
- Keep domain vocabulary upstream in domain/stage/pipeline crates.
- Prefer workspace dependency declarations when the dependency is already listed at the workspace root.

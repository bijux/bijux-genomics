# Ownership

This page defines who owns what at the crate level. Ownership means: source of truth for data models, defaults, contracts, and invariants.

## Crate ownership

- bijux-core: core types, shared data contracts, serialization, and cross-cutting utilities.
- bijux-domain-*: domain models and validation for each domain (fastq, bam, vcf, dummy). No execution.
- bijux-stages-fastq: planning, defaults, and artifacts contracts for FASTQ stages. Owns stage plans and output schemas.
- bijux-engine: execution, observation, tool invocation, runtime metrics, and artifact materialization. Owns execution semantics.
- bijux-cli: UX adapter only. Parses args, displays errors, and delegates to engine/stages.
- bijux-analyze: analysis and reporting over artifacts and metrics. Pure consumer.
- bijux-bench: benchmarking orchestration for experiments. Pure consumer.
- bijux-environment: runtime environment, tool images, and platform capabilities.

## Ownership guardrails

- Planning and defaults live in stages-* crates, not the CLI or engine.
- Tool execution lives in bijux-engine only.
- Domain types do not cross the engine API boundary.
- CLI must not contain tool IDs or execution logic.

# Bijux boundaries contract

This document defines ownership and allowed dependencies. Treat it as an API contract.

## OWNERSHIP
- IDs (PipelineId/StageId/ToolId/MetricId): bijux-core
- Defaults/profiles: bijux-pipelines
- Param schemas: bijux-domain-*
- Metric semantics: bijux-domain-* (definitions) + bijux-analyze (evaluation rules)
- Artifact layout: bijux-infra (path builders) + bijux-runtime (recording)
- Report schema/rendering: bijux-api (schema) + bijux-cli (rendering)

## bijux-core
- Owns: shared data types, schemas, hashing, stage plans.
- Does not own: domain rules, planning heuristics, execution, I/O side effects.
- Allowed deps: std + infra + small third‑party utilities only.

## bijux-domain-*
- Owns: domain models, validation, benchmark records, domain policies.
- Does not own: execution, environment discovery, CLI UX.
- Allowed deps: core + infra + domain‑local dependencies.

## bijux-stages-*
- Owns: stage planning (defaults, outputs, contracts), registry.
- Does not own: execution, CLI UX, environment detection.
- Allowed deps: core + domain + infra.

## bijux-planner-*
- Owns: pipeline planning and tool selection for a domain.
- Does not own: execution, CLI UX, environment detection.
- Allowed deps: core + stages + pipelines + selection + infra.

## bijux-api
- Owns: orchestration surface (plan/run/report/bench) and stable public API.
- Does not own: stage specs, domain rules, CLI UX.
- Allowed deps: core + planner + engine + runner + env + analyze + pipelines + infra + runtime.

## bijux-engine
- Owns: pure execution scheduling, artifact helpers.
- Does not own: CLI argument parsing, domain business rules, execution adapters.
- Allowed deps: core + runtime + infra.

## bijux-runner-*
- Owns: execution adapters (docker/local), replay.
- Does not own: CLI UX, orchestration, execution scheduling.
- Allowed deps: engine + core + env + infra.

## bijux-environment
- Owns: runner/platform discovery, image resolution.
- Does not own: domain planning or CLI.
- Allowed deps: core + infra.

## bijux-cli
- Owns: UX, argument parsing, user‑facing error mapping.
- Does not own: planning logic, execution, environment inspection.
- Allowed deps: api + clap + logging only.

## bijux-analyze / bijux-benchmark
- Owns: analysis/bench evaluation and reporting.
- Does not own: planning, execution, CLI UX.
- Allowed deps: core + domain + infra + runtime (no engine/stages).

## Executable dependency map
```boundaries
bijux-core: bijux-infra
bijux-runtime: bijux-core bijux-infra
bijux-domain-fastq: bijux-core bijux-infra
bijux-domain-bam: bijux-core bijux-infra
bijux-stages-fastq: bijux-core bijux-domain-fastq bijux-infra
bijux-stages-bam: bijux-core bijux-domain-bam bijux-infra
bijux-planner-fastq: bijux-core bijux-stages-fastq bijux-pipelines bijux-infra
bijux-planner-bam: bijux-core bijux-stages-bam bijux-infra
bijux-engine: bijux-core bijux-runtime bijux-infra
bijux-runner: bijux-core bijux-engine bijux-environment bijux-infra
bijux-environment: bijux-core bijux-infra
bijux-pipelines: bijux-core bijux-domain-fastq bijux-domain-bam
bijux-analyze: bijux-core bijux-domain-fastq bijux-domain-bam bijux-infra bijux-runtime
bijux-benchmark-model: bijux-analyze
bijux-benchmark: bijux-core bijux-analyze bijux-benchmark-model bijux-infra bijux-runtime
bijux-api: bijux-core bijux-engine bijux-runner bijux-environment bijux-analyze bijux-pipelines bijux-infra bijux-planner-fastq bijux-planner-bam bijux-runtime
bijux-cli: bijux-api
bijux: bijux-api
```

## Review checklist (determinism)
- Any SQL query that selects a single/latest record MUST use `ORDER BY` on a stable key
  (e.g., `record_id`, `inserted_at`) + `LIMIT 1`.
- Add/extend a determinism test when introducing new "latest record" queries.

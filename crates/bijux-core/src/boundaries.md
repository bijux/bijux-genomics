# Bijux boundaries contract

This document defines ownership and allowed dependencies. Treat it as an API contract.

## OWNERSHIP
- IDs (PipelineId/StageId/ToolId/MetricId): bijux-core
- Defaults/profiles: bijux-pipelines
- Param schemas: bijux-domain-*
- Metric semantics: bijux-domain-* (definitions) + bijux-analyze (evaluation rules)
- Artifact layout: bijux-infra (path builders) + bijux-runtime (recording)
- Report schema/rendering: bijux-api (schema) + bijux-cli (rendering)
- Stage plan/plugin contracts: bijux-stage-contract

## bijux-core
- Owns: shared data types, schemas, hashing, execution graph.
- Does not own: domain rules, planning heuristics, execution, I/O side effects.
- Allowed deps: std + infra + small third‑party utilities only.

## bijux-stage-contract
- Owns: stage plans, execution plans, and stage plugin contracts.
- Does not own: execution, domain semantics, environment resolution.
- Allowed deps: core + small third‑party utilities only.

## bijux-domain-*
- Owns: domain models, validation, benchmark records, domain policies.
- Does not own: execution, environment discovery, CLI UX.
- Allowed deps: core + infra + domain‑local dependencies.

## bijux-stages-*
- Owns: stage defaults, outputs, and contracts (no registries).
- Does not own: execution, CLI UX, environment detection.
- Allowed deps: core + domain + infra.

## bijux-planner-*
- Owns: pipeline planning, stage registries, and tool selection for a domain.
- Does not own: execution, CLI UX, environment detection.
- Allowed deps: core + stages + pipelines + selection + infra.

## bijux-api
- Owns: orchestration surface (plan/run/report/bench) and stable public API.
- Does not own: stage specs, domain rules, CLI UX.
- Allowed deps: core + planner + engine + runner + env + analyze + pipelines + infra + runtime.

## bijux-infra
- Owns: generic IO, formats, logging, and utility helpers.
- Does not own: domain semantics, planning, or execution.
- Allowed deps: policies (dev-only guardrails).

## bijux-policies
- Owns: guardrail policies, workspace audits, and architectural checks.
- Does not own: production logic or runtime behavior.
- Allowed deps: none (workspace deps are forbidden).

## bijux-engine
- Owns: pure execution scheduling, artifact helpers.
- Does not own: CLI argument parsing, domain business rules, execution adapters.
- Allowed deps: core + infra.

## bijux-runner-*
- Owns: execution adapters (docker/local), replay.
- Does not own: CLI UX, orchestration, execution scheduling.
- Allowed deps: core + env + infra.

## bijux-environment
- Owns: runner/platform discovery, image resolution.
- Does not own: domain planning or CLI.
- Allowed deps: core + infra.

## bijux-environment-qa
- Owns: image QA scenarios, datasets, behavioral tests.
- Does not own: runtime orchestration or planning.
- Allowed deps: env + analyze + core + infra + runtime.

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
bijux-core: bijux-infra bijux-policies
bijux-infra: bijux-policies
bijux-policies:
bijux-stage-contract: bijux-core
bijux-runtime: bijux-core bijux-infra
bijux-domain-fastq: bijux-core bijux-infra bijux-policies
bijux-domain-bam: bijux-core bijux-infra bijux-policies
bijux-stages-fastq: bijux-core bijux-domain-fastq bijux-stage-contract bijux-infra bijux-runtime bijux-planner-fastq bijux-policies
bijux-stages-bam: bijux-core bijux-domain-bam bijux-stage-contract bijux-infra bijux-policies
bijux-planner-fastq: bijux-core bijux-stage-contract bijux-domain-fastq bijux-domain-bam bijux-stages-fastq bijux-pipelines bijux-infra bijux-policies
bijux-planner-bam: bijux-core bijux-stage-contract bijux-domain-bam bijux-stages-bam bijux-pipelines bijux-infra bijux-policies
bijux-engine: bijux-core bijux-infra bijux-policies
bijux-runner: bijux-core bijux-environment bijux-infra bijux-policies
bijux-environment: bijux-core bijux-infra bijux-runtime bijux-policies
bijux-environment-qa: bijux-environment bijux-analyze bijux-core bijux-domain-fastq bijux-infra bijux-runtime bijux-policies
bijux-pipelines: bijux-core bijux-domain-fastq bijux-domain-bam bijux-policies
bijux-analyze: bijux-core bijux-domain-fastq bijux-domain-bam bijux-infra bijux-runtime bijux-pipelines bijux-planner-fastq bijux-planner-bam bijux-policies
bijux-benchmark-model: bijux-analyze
bijux-benchmark: bijux-core bijux-analyze bijux-benchmark-model bijux-infra bijux-runtime bijux-policies
bijux-api: bijux-core bijux-stage-contract bijux-engine bijux-runner bijux-environment bijux-environment-qa bijux-analyze bijux-benchmark bijux-pipelines bijux-infra bijux-planner-fastq bijux-planner-bam bijux-runtime bijux-policies
bijux-cli: bijux-api
bijux: bijux-api bijux-core bijux-environment bijux-environment-qa bijux-infra bijux-stage-contract bijux-policies
```

## Review checklist (determinism)
- Any SQL query that selects a single/latest record MUST use `ORDER BY` on a stable key
  (e.g., `record_id`, `inserted_at`) + `LIMIT 1`.
- Add/extend a determinism test when introducing new "latest record" queries.

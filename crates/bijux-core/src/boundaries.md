# Bijux boundaries contract

This document defines ownership and allowed dependencies. Treat it as an API contract.

## bijux-core
- Owns: shared data types, schemas, hashing, observability contracts, stage plans.
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

## bijux-api
- Owns: orchestration surface (plan/run/report/bench) and stable public API.
- Does not own: stage specs, domain rules, CLI UX.
- Allowed deps: core + stages + engine + env + analyze + pipelines + infra.

## bijux-engine
- Owns: execution, observation, artifact emission, telemetry, failure mapping.
- Does not own: CLI argument parsing, domain business rules.
- Allowed deps: core + stages + environment + infra.

## bijux-env-runtime / bijux-env-builder
- Owns: runner/platform discovery, image resolution.
- Does not own: domain planning or CLI.
- Allowed deps: core + infra.

## bijux-cli
- Owns: UX, argument parsing, user‑facing error mapping.
- Does not own: planning logic, execution, environment inspection.
- Allowed deps: api + clap + logging only.

## bijux-analyze / bijux-bench
- Owns: analysis/bench evaluation and reporting.
- Does not own: planning, execution, CLI UX.
- Allowed deps: core + domain + infra (no engine/stages).

## Executable dependency map
```boundaries
bijux-core: bijux-infra
bijux-domain-fastq: bijux-core bijux-infra
bijux-domain-bam: bijux-core bijux-infra
bijux-domain-vcf: bijux-core bijux-infra
bijux-stages-fastq: bijux-core bijux-domain-fastq bijux-infra
bijux-stages-bam: bijux-core bijux-domain-bam bijux-infra
bijux-engine: bijux-core bijux-stages-fastq bijux-stages-bam bijux-env-runtime bijux-env-builder bijux-infra
bijux-env-runtime: bijux-core bijux-infra
bijux-env-builder: bijux-core bijux-infra
bijux-pipelines: bijux-core bijux-domain-fastq bijux-domain-bam bijux-domain-vcf
bijux-analyze: bijux-core bijux-domain-fastq bijux-domain-bam bijux-domain-vcf bijux-infra
bijux-bench: bijux-core bijux-analyze bijux-engine bijux-infra
bijux-api: bijux-core bijux-engine bijux-env-runtime bijux-env-builder bijux-analyze bijux-stages-fastq bijux-stages-bam bijux-domain-fastq bijux-domain-bam bijux-domain-vcf bijux-pipelines bijux-infra
bijux-cli: bijux-api
bijux: bijux-api
```

## Review checklist (determinism)
- Any SQL query that selects a single/latest record MUST use `ORDER BY` on a stable key
  (e.g., `record_id`, `inserted_at`) + `LIMIT 1`.
- Add/extend a determinism test when introducing new "latest record" queries.

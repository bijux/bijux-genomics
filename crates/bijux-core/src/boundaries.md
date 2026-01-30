# Bijux boundaries contract

This document defines ownership and allowed dependencies. Treat it as an API contract.

## bijux-core
- Owns: shared data types, schemas, hashing, observability contracts, stage plans.
- Does not own: domain rules, planning heuristics, execution, I/O side effects.
- Allowed deps: std + small third‑party utilities only.

## bijux-domain-*
- Owns: domain models, validation, benchmark records, domain policies.
- Does not own: execution, environment discovery, CLI UX.
- Allowed deps: core + domain‑local dependencies.

## bijux-stages-*
- Owns: stage planning (defaults, outputs, contracts), registry.
- Does not own: execution, CLI UX, environment detection.
- Allowed deps: core + domain.

## bijux-engine
- Owns: execution, observation, artifact emission, telemetry, failure mapping.
- Does not own: CLI argument parsing, domain business rules.
- Allowed deps: core + stages + environment.

## bijux-environment
- Owns: runner/platform discovery, image resolution.
- Does not own: domain planning or CLI.
- Allowed deps: core.

## bijux-cli
- Owns: UX, argument parsing, user‑facing error mapping.
- Does not own: planning logic, execution, environment inspection.
- Allowed deps: core + stages + engine.

## bijux-analyze / bijux-bench
- Owns: analysis/bench evaluation and reporting.
- Does not own: planning, execution, CLI UX.
- Allowed deps: core + domain (no engine/stages).

## Review checklist (determinism)
- Any SQL query that selects a single/latest record MUST use `ORDER BY` on a stable key
  (e.g., `record_id`, `inserted_at`) + `LIMIT 1`.
- Add/extend a determinism test when introducing new "latest record" queries.

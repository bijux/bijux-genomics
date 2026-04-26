# Boundary

Owner: `bijux-dna-api`

This crate owns the stable programmatic API that composes planner, runtime,
runner, report, environment, analyzer, benchmark, policy-audit, and domain
contracts for v1 callers.

## Allowed Inputs

- Typed request structs from `src/surface/request_contracts.rs`.
- Execution graphs, stage plans, run specs, profiles, runtime manifests, and
  analyzer/report facts from lower-level crates.
- Repository-scoped configuration and registry views resolved through
  `src/support/workspace/`.
- Reference and tool metadata resolved through API-local support modules.

## Allowed Effects

- Write run, dry-run, report, summary, audit, recovery, and manifest artifacts
  only under caller-declared output roots.
- Read workspace configuration, registries, snapshots, run artifacts, and
  reference locks needed to answer API requests.
- Call typed runner/runtime/environment/analyzer APIs.
- Hash files for provenance when the workflow declares those artifacts.

## Forbidden Effects

- Ad hoc shell execution or direct process/container spawning outside the
  runner/runtime boundary.
- Undeclared writes outside caller-declared output roots.
- Network access as part of API request handling.
- Hidden global state that changes response shape for the same declared inputs.
- Re-exporting lower-level crates wholesale as public API.

## Dependency Direction

Allowed normal dependencies are the crates needed to compose API workflows:

- `bijux-dna-core`
- `bijux-dna-domain-bam`
- `bijux-dna-domain-fastq`
- `bijux-dna-stage-contract`
- `bijux-dna-engine`
- `bijux-dna-environment`
- `bijux-dna-analyze`
- `bijux-dna-bench`
- `bijux-dna-planner-fastq`
- `bijux-dna-planner-bam`
- `bijux-dna-runner`
- `bijux-dna-pipelines`
- `bijux-dna-infra`
- `bijux-dna-runtime`

Test-only policy and guardrail helpers belong in dev-dependencies unless source
code uses them for API-owned audit/report output.

## Determinism

Planning, dry-run, explainability, and schema materialization must be
deterministic for the same declared inputs. Execution status can reflect runtime
state, but response fields must keep stable meaning and shape.

## Boundary Tests

`tests/boundaries/architecture.rs` protects the root layout, docs allowance,
source namespace layout, and test documentation location.

`tests/boundaries/guardrails.rs` and `tests/guardrails.rs` run shared policy
checks for the crate.

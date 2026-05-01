# Crate Responsibility Matrix

Owner: Architecture  
Scope: workspace crate ownership, forbidden drift, and public entrypoints  
Last reviewed: 2026-04-30  
Contract version: v1

## Purpose

Give contributors one place to answer four questions before they edit code:

1. which crate owns this responsibility;
2. which responsibilities are forbidden there;
3. which public surface is allowed to expose the behavior;
4. which dependency direction keeps the workspace acyclic.

The machine-readable companion is
[../../configs/ci/crate-boundaries.toml](../../configs/ci/crate-boundaries.toml).

## Dependency direction

The intended direction is:

`foundation -> domain -> stage-contract -> planners -> stages -> runtime -> runner -> engine -> api -> cli`

Parallel consumers that must not own execution truth:

- `bijux-dna-analyze`
- `bijux-dna-bench`
- `bijux-dna-bench-model`
- `bijux-dna-science`
- `bijux-dna-testkit`
- `bijux-dna-dev`

## Matrix

| Crate | Allowed responsibilities | Forbidden responsibilities | Public surface | Dependency direction |
| --- | --- | --- | --- | --- |
| `bijux-dna` | CLI argument parsing, API delegation, human/operator rendering | planner truth, runtime truth, product execution internals | `src/lib.rs`, `docs/PUBLIC_API.md` | terminal adapter over `bijux-dna-api` |
| `bijux-dna-analyze` | evidence loading, fact extraction, report comparison | execution orchestration, backend invocation, planner defaults ownership | `src/lib.rs`, `docs/PUBLIC_API.md` | consumes artifacts and evidence |
| `bijux-dna-api` | stable request/response contracts, plan/run/report orchestration, preflight helpers | CLI-only formatting, shell spawning, hidden scientific defaults | `src/lib.rs`, `docs/PUBLIC_API.md` | bridges planning/runtime into versioned APIs |
| `bijux-dna-bench` | benchmark orchestration and report shaping | domain truth ownership, planner truth ownership | `src/lib.rs`, `docs/PUBLIC_API.md` | consumes analyze/runtime evidence |
| `bijux-dna-bench-model` | typed comparison models and benchmark diffs | runtime execution, filesystem authority | `src/lib.rs`, `docs/PUBLIC_API.md` | pure analysis support |
| `bijux-dna-core` | stable IDs, canonicalization, shared manifest/model types | domain-specific policy, runtime effects, command execution | `src/lib.rs`, `docs/PUBLIC_API.md` | foundational leaf |
| `bijux-dna-db-ena` | ENA download planning, metadata normalization, offline fixtures | pipeline truth, benchmark publication truth | `src/lib.rs`, `docs/PUBLIC_API.md` | support crate |
| `bijux-dna-db-ref` | reference compatibility and reference asset resolution helpers | planner orchestration, runtime execution | `src/lib.rs`, `docs/PUBLIC_API.md` | support crate |
| `bijux-dna-dev` | governance commands, inventory/drift reports, repo-local automation | domain truth, pipeline truth, runtime truth | `docs/COMMANDS.md`, `src/catalog/ops.rs` | governance consumer only |
| `bijux-dna-domain-bam` | BAM stage vocabulary, BAM metrics/invariants, typed BAM truth | run layout, backend spawning, API response shaping | `src/lib.rs`, `docs/PUBLIC_API.md` | domain layer |
| `bijux-dna-domain-compiler` | compile domain YAML into typed registries and indexes | execution logic, planner defaults, CLI policy | `src/lib.rs`, `docs/PUBLIC_API.md` | domain compiler |
| `bijux-dna-domain-fastq` | FASTQ stage vocabulary, FASTQ invariants, typed FASTQ truth | run layout, backend spawning, API response shaping | `src/lib.rs`, `docs/PUBLIC_API.md` | domain layer |
| `bijux-dna-domain-vcf` | VCF stage vocabulary, cohort/reference invariants, typed VCF truth | run layout, backend spawning, API response shaping | `src/lib.rs`, `docs/PUBLIC_API.md` | domain layer |
| `bijux-dna-engine` | explicit-plan execution orchestration and lifecycle coordination | hidden planning, domain catalog truth, CLI parsing | `src/lib.rs`, `docs/PUBLIC_API.md` | engine over runtime/runner |
| `bijux-dna-environment` | runtime platform resolution, image/platform discovery, environment contracts | planner defaults, scientific policy ownership | `src/lib.rs`, `docs/PUBLIC_API.md` | execution support |
| `bijux-dna-environment-qa` | environment QA assertions and readiness checks | production runtime dependencies from product crates | `src/lib.rs`, `docs/PUBLIC_API.md` | QA-only support |
| `bijux-dna-infra` | generic filesystem/config helpers and low-level utilities | domain semantics, planner/runtime policy truth | `src/lib.rs`, `docs/PUBLIC_API.md` | foundational leaf |
| `bijux-dna-pipelines` | workflow/profile composition, defaults ledgers, template-level invariants | backend execution, CLI policy ownership | `src/lib.rs`, `docs/PUBLIC_API.md` | pipeline composition |
| `bijux-dna-planner-bam` | BAM stage selection, plan assembly, explain output | backend spawning, run persistence, CLI formatting | `src/lib.rs`, `docs/PUBLIC_API.md` | planner layer |
| `bijux-dna-planner-fastq` | FASTQ stage selection, plan assembly, explain output | backend spawning, run persistence, CLI formatting | `src/lib.rs`, `docs/PUBLIC_API.md` | planner layer |
| `bijux-dna-planner-vcf` | VCF stage selection, plan assembly, explain output | backend spawning, run persistence, CLI formatting | `src/lib.rs`, `docs/PUBLIC_API.md` | planner layer |
| `bijux-dna-policies` | CI/test policy enforcement, workspace structure checks, contract references | runtime product behavior, output mutation | `src/lib.rs`, `docs/PUBLIC_API.md` | governance leaf |
| `bijux-dna-runner` | command/container execution, backend adapters, exit-status capture | plan selection, domain interpretation, report truth | `src/lib.rs`, `docs/PUBLIC_API.md` | runner below engine |
| `bijux-dna-runtime` | run layout, manifest persistence, runtime records and observability | planner selection, CLI formatting, backend spawning | `src/lib.rs`, `docs/PUBLIC_API.md` | runtime below engine |
| `bijux-dna-science` | scientific reference compilation and evidence-facing science docs | execution orchestration, hidden runtime policy | `src/lib.rs`, `docs/PUBLIC_API.md` | science consumer |
| `bijux-dna-stage-contract` | stable stage-plan/input/output/report contracts | planner orchestration, runtime side effects | `src/lib.rs`, `docs/PUBLIC_API.md` | contract layer |
| `bijux-dna-stages-bam` | BAM stage invocation builders, parsing contracts, typed outputs | planner graph assembly, runner orchestration | `src/lib.rs`, `docs/PUBLIC_API.md` | stage layer |
| `bijux-dna-stages-fastq` | FASTQ stage invocation builders, parsing contracts, typed outputs | planner graph assembly, runner orchestration | `src/lib.rs`, `docs/PUBLIC_API.md` | stage layer |
| `bijux-dna-stages-vcf` | VCF stage invocation builders, parsing contracts, typed outputs | planner graph assembly, runner orchestration | `src/lib.rs`, `docs/PUBLIC_API.md` | stage layer |
| `bijux-dna-testkit` | fixture helpers, snapshot normalization, test support | production runtime truth, product orchestration | `src/lib.rs`, `docs/PUBLIC_API.md` | test-support leaf |

## Enforcement

- [BOUNDARY_MAP.md](BOUNDARY_MAP.md) remains the executable dependency authority.
- [CRATE_BOUNDARY_CONTRACTS.md](CRATE_BOUNDARY_CONTRACTS.md) defines required boundary fields.
- `crates/bijux-dna-policies/tests/contracts/tooling/governance_core/crate_responsibility_matrix_policy.rs`
  requires every workspace crate to appear in the machine-readable matrix.

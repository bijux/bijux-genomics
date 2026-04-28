# Crate Boundary Contracts

Owner: Architecture
Scope: Workspace crate family boundaries
Last reviewed: 2026-04-26
Contract version: v1

## Purpose
Define the common required fields and family-specific authority for crate `BOUNDARY.md` files.

## Scope
- Workspace crate-family boundary documents and their required fields.
- The family-level ownership rules enforced by policy tests.

## Non-goals
- Replacing crate-local `BOUNDARY.md` files.
- Restating every crate's implementation structure or public API.

## Contracts
- Every governed crate boundary document must expose the required fields listed below.
- Family-level rules define the minimum contract each crate must refine locally.

## Required fields
Every crate root `BOUNDARY.md` must declare:
- `Owner:`
- `Scope:`
- `Allowed inputs:`
- `Forbidden dependencies:`
- `Forbidden effects:`
- `Validation command:`

## Family contracts
| Crate family | Owner | Allowed inputs | Forbidden dependencies | Forbidden effects | Validation command |
| --- | --- | --- | --- | --- | --- |
| `bijux-dna` | CLI | API responses, registry/config reads, current working directory | engine internals, runner internals, direct product execution | undeclared file writes, network access, process spawning | `cargo test -p bijux-dna --no-default-features` |
| `bijux-dna-api` | API | typed planner/runtime/environment contracts | CLI adapters as a required runtime dependency | direct shell/container process spawning | `cargo test -p bijux-dna-api --no-default-features` |
| `bijux-dna-analyze` | Analyze | produced run/report artifacts | runner, engine internals, CLI adapters | product execution, generated config mutation | `cargo test -p bijux-dna-analyze --no-default-features` |
| `bijux-dna-core` | Core | typed IDs, manifests, shared models | planner, runner, engine, CLI, environment crates | filesystem, process, network effects | `cargo test -p bijux-dna-core --no-default-features` |
| `bijux-dna-domain-*` | Domain | authored domain vocabularies and scientific constraints | runtime, runner, engine, API, CLI crates | execution, generated config writes | `cargo test -p <crate> --no-default-features` |
| `bijux-dna-domain-compiler` | Domain compiler | domain source files and compiler options | runner, CLI, engine internals | product execution and network access | `cargo test -p bijux-dna-domain-compiler --no-default-features` |
| `bijux-dna-planner-*` | Planner | domain/stage contracts, profiles, registry views | runner, CLI adapters, environment probes | process spawning and product execution | `cargo test -p <crate> --no-default-features` |
| `bijux-dna-stages-*` | Stages | typed stage contracts and fixture observations | CLI, API, engine, planner orchestration | product execution outside declared fixture tests | `cargo test -p <crate> --no-default-features` |
| `bijux-dna-runner` | Runner | explicit tool invocation requests | planner/domain semantics, CLI adapters | network access unless declared by runtime policy | `cargo test -p bijux-dna-runner --no-default-features` |
| `bijux-dna-runtime` | Runtime | execution plans, runner responses, manifest layouts | CLI adapters and planner selection logic | undeclared writes outside run layouts | `cargo test -p bijux-dna-runtime --no-default-features` |
| `bijux-dna-infra` | Infra | generic filesystem/config primitives | domain semantics, planner/runtime orchestration | domain-specific effects and process spawning | `cargo test -p bijux-dna-infra --no-default-features` |
| `bijux-dna-testkit` | Testkit | test fixtures and sanitizers | product/runtime crates as production dependencies | production execution and network access | `cargo test -p bijux-dna-testkit --no-default-features` |
| `bijux-dna-policies` | Policies | repository files, docs, configs, fixtures | product execution crates as required runtime dependencies | mutating source, generated configs, snapshots, or network state | `cargo test -p bijux-dna-policies --no-default-features` |

## Enforcement
- Dependency edges are enforced by [BOUNDARY_MAP.md](BOUNDARY_MAP.md).
- Required crate `BOUNDARY.md` fields are enforced by
  [../../crates/bijux-dna-policies/tests/contracts/tooling/docs/boundary_docs_policy.rs](../../crates/bijux-dna-policies/tests/contracts/tooling/docs/boundary_docs_policy.rs).

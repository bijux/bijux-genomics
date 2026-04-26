# Architecture

`bijux-dna-core` is the lowest shared model crate in `bijux-genomics`. It owns
stable contract types, canonical identifiers, hashing, deterministic
serialization, metric contracts, and the curated prelude surface.

## Root Layout

- `Cargo.toml` declares the pure core dependency graph.
- `README.md` is the only root documentation file.
- `docs/` contains the 10 authoritative crate docs.
- `src/` contains the library implementation.
- `tests/` contains boundary, contract, schema, semantic, fixture, snapshot, and
  guardrail coverage.

## Source Map

- `src/contract/` owns execution graphs, run records, tooling contracts, and
  canonical JSON helpers.
- `src/foundation/` owns low-level shared models and deterministic helpers used
  by core contracts.
- `src/id_catalog/` owns canonical pipeline, stage, and tool identifiers.
- `src/ids/` owns typed identifiers, parsing rules, and symbolic validation.
- `src/metrics/` owns metric identifiers, derived metrics, schema lookup, and
  registry constants.
- `src/prelude/` exposes the stable ergonomic import surface.
- `src/public_api/` mirrors the stable public modules through explicit
  namespaces.

## Test Map

- `tests/boundaries.rs` checks source layout, dependency graph, layering, docs
  placement, and guardrails.
- `tests/contracts.rs` checks execution, identity, run metadata, and public
  surface contracts.
- `tests/schemas.rs` checks schema and public module snapshots.
- `tests/semantics.rs` checks identifier, metric, and input-assessment
  semantics.
- `tests/fixtures/` and `tests/snapshots/` hold governed test data only.

## Boundaries

Core must not depend on domain, planner, API, engine, runner, or CLI crates. It
may define pure contracts consumed by those crates. The only documented
filesystem exception is `foundation::input_assessment`, which discovers and
writes typed FASTQ assessment contracts.

## Dependency Direction

Downstream crates consume core contracts. Core does not call downstream crates,
select tools, run workflows, or interpret domain-specific stage policy.

## Layout Rules

- Keep `src/contract/` for shared contracts, not runtime orchestration.
- Keep `src/foundation/` pure except for the documented typed input-assessment
  filesystem exception.
- Keep identifier catalogs under `src/id_catalog/`; do not scatter canonical
  IDs through downstream crates.
- Keep prelude exports curated and mirrored by public API tests.
- Update `tests/boundaries/architecture_tree.rs` and this document together when
  the source or test tree changes intentionally.

## Command Inventory

`docs/COMMANDS.md` lists the library operations this crate manages. Keep command
entries aligned with public modules and contract tests.

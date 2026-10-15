# bijux-dna-infra Boundary

Owner: deterministic infrastructure primitives.
Scope: generic filesystem IO, hashing, retry, paths, logging bootstrap, temp directories, file
locking, and config-compatible formats.
Allowed inputs: paths, bytes, serialized config payloads, deterministic fixtures.
Forbidden dependencies: domain semantics, planner/runtime orchestration, CLI adapters.
Forbidden effects: process spawning, network access, domain-specific writes.
Validation command:
`TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-infra --no-default-features --test boundaries`

## Belongs Here

- Deterministic path construction that only joins confined path segments.
- Generic filesystem reads, writes, removals, temporary directories, file locks, and file digests.
- Physical run/attempt layout, publish/lock and archive/rollback receipt contracts. These schemas
  describe filesystem custody and do not own biological, stage, or pipeline semantics.
- Retry policy, backoff math, and clock abstraction.
- Logging bootstrap for callers that enable the `tracing` feature.
- JSON, TOML, and optional YAML helpers for config-compatible payloads.

## Scope Rules

Belongs here: deterministic helpers and generic filesystem utilities.
Does not belong here: domain knowledge, biological/stage contract schemas, command routing, or orchestration.

## Does Not Belong Here

- Biological domain models, stage/tool catalogs, pipeline semantics, or their contract schema ownership.
- CLI routing, planner selection, runtime execution, or process orchestration.
- Semantic contract canonicalization, which belongs to `bijux-dna-core`. Physical owner-root
  overlap admission may resolve existing filesystem ancestors before comparing evidence roots.
- Shared governance policy ownership.

## No Domain Semantics

Forbidden code includes imports from domain, stage, planner, runner, API, database, analysis,
environment, science, and benchmark crates; defining stage/tool catalogs; and embedding product
workflow IDs in infra helpers.

## Dependency Direction

This crate is a lower-level utility crate. Other crates may consume its stable helpers, but infra
must not depend on domain, planner, pipeline, runner, API, CLI, database, analysis, benchmark, or
environment crates.

## Effects

Allowed runtime effects are generic filesystem effects initiated by callers. Default tests must stay
offline and must not execute host commands.

## Family Contract

The family-level contract is indexed in
[docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md](../../../docs/10-architecture/CRATE_BOUNDARY_CONTRACTS.md).

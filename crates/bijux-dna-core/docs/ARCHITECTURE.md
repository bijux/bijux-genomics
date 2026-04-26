# bijux-dna-core Architecture

`bijux-dna-core` is the lowest shared model crate in `bijux-genomics`. It owns
stable contract types, canonical identifiers, metrics contracts, deterministic
serialization, hashing, and a curated public import surface.

## Source Tree

```text
src/
├── lib.rs
├── contract/
│   ├── canonical.rs
│   ├── execution/
│   ├── run/
│   ├── tooling/
│   └── version.rs
├── foundation/
│   ├── cache.rs
│   ├── canonical.rs
│   ├── command/
│   ├── errors.rs
│   ├── hashing.rs
│   ├── input_assessment.rs
│   ├── invariants.rs
│   ├── measure.rs
│   └── version.rs
├── id_catalog/
│   ├── pipeline/
│   ├── stage/
│   └── tool/
├── ids/
│   ├── domain_model.rs
│   ├── parsing/
│   └── typed/
├── metrics/
├── prelude/
└── public_api/
```

## Module Roles

- `contract/` owns serialized execution, run, provenance, tooling, selection,
  versioning, and canonical JSON contract entrypoints.
- `foundation/` owns crate-local implementation helpers that support public
  contracts: command specs, container image refs, canonicalization, hashing,
  input assessment, errors, cache keys, invariant records, and measurement
  records.
- `id_catalog/` owns canonical pipeline, stage, and tool ids partitioned by
  workflow/domain.
- `ids/` owns typed id wrappers, parsing rules, symbolic id validation, and
  domain/library metadata models.
- `metrics/` owns metric ids, derived metric parsing, schema lookup, registry
  constants, and metric payload contracts.
- `prelude/` groups imports by source area and exposes the stable ergonomic
  surface through `stable_surface.rs`.
- `public_api/` mirrors the stable public modules through explicit namespaces:
  `contracts`, `catalog`, `identity`, `metrics`, and `ergonomics`.

## Test Tree

```text
tests/
├── boundaries.rs
├── boundaries/
├── contracts.rs
├── contracts/
├── fixtures/
├── guardrails.rs
├── schemas.rs
├── schemas/
├── semantics.rs
├── semantics/
└── snapshots/
```

Boundary tests lock the crate root, docs allowance, source tree, test tree,
source layering, and scope. Contract tests cover execution, identity, canonical
JSON, run index, run metadata, selection, and public surface behavior. Schema
tests lock docs and public surfaces. Semantic tests cover identifiers, metrics,
and input assessment.

## Dependency Shape

Normal dependencies must stay low-level and contract-facing:

- `serde`, `serde_json`, `thiserror`, `sha2`, and `regex` support stable
  contract serialization, errors, hashing, and validation.
- `chrono` and `walkdir` support input assessment records.
- `tempfile` supports the local temp-file rename used by input assessment
  persistence.

This crate must not depend on API, CLI, planner, runner, engine, runtime,
environment, domain, stage, analyze, or benchmark crates. Any dependency that
pulls orchestration or product behavior into core is a boundary violation.

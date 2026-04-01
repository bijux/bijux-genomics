# Architecture

## Tree
- `src/public_api/` mirrors the curated stable surface through dedicated `contracts/`, `catalog/`,
  `identity/`, `metrics/`, and `ergonomics/` namespaces.
- `src/contract/` owns canonical serialization, execution contracts, run contracts, tooling contracts, and versioning.
- `src/id_catalog/` owns canonical stage, pipeline, and tool identifiers through `pipeline/`, `stage/`, and `tool/` namespaces.
- `src/ids/` owns typed identifiers, parsing rules, and domain/library models through `typed/` and `parsing/` namespaces.
- `src/metrics/` owns metrics types, schemas, and registry semantics.
- `src/prelude/` groups stable imports by contract, catalog, identity, foundation, and metrics
  source areas, with `stable_surface.rs` owning the aggregated export surface.
- `src/foundation/` remains crate-internal support for hashing, errors, command specs, invariants,
  and input assessment, with `command/` separating command templates from container image refs.

## Data flow
1. `contract`, `ids`, `id_catalog`, and `metrics` define canonical workspace contracts.
2. `prelude` and `public_api` expose stable import surfaces for downstream crates, with
   `public_api` partitioned into explicit contract, catalog, identity, metrics, and ergonomics
   namespaces.
3. `foundation` supports those public contracts without taking on runtime behavior.

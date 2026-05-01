# Public API

`bijux-dna-core` keeps a small public module set so downstream crates can depend
on stable contracts without importing implementation layout by accident.

## Public Modules

| Module | Role |
| --- | --- |
| `contract` | Serialized execution, run, tooling, selection, version, and canonical JSON contracts. |
| `id_catalog` | Canonical pipeline, stage, and tool id constants. |
| `ids` | Typed identifiers, id parsing, symbolic validation, and domain model records. |
| `metrics` | Metric ids, schemas, registry lookup, derived metric parsing, and metric payloads. |
| `prelude` | Stable ergonomic import groups for downstream crates. |
| `public_api` | Curated mirror partitioned into `contracts`, `catalog`, `identity`, `metrics`, and `ergonomics`. |

## Extension Rules

1. Add new types under an existing public module whenever the owner is clear.
2. Keep helpers `pub(crate)` unless downstream crates need the type or function.
3. Prefer `prelude` re-exports for ergonomic access and `public_api` mirrors for
   stable curated access.
4. Do not add a new root public module without updating `README.md`,
   `docs/ARCHITECTURE.md`, this file, schema tests, and public-surface
   snapshots.
5. Do not expose orchestration, runner, planner, CLI, API, or product behavior
   through core public modules.
6. If a public helper is a callable operation rather than a data type or
   constructor, add it to `docs/COMMANDS.md` in the same change set.
7. If a public helper reads or writes the filesystem, document the effect in
   `docs/BOUNDARY.md` and cover the behavior in the closest semantic or
   contract test.

## Enforcement

- `tests/schemas/public_module_tree.rs` locks the root public module tree.
- `tests/schemas/public_surface.rs` and
  `tests/schemas/public_surface_lock.rs` lock the curated public surface.
- `tests/contracts/identity/prelude_snapshot.rs` locks prelude ergonomics.
- `tests/schemas/docs_public_api.rs` locks the managed operation inventory and
  the README docs allowance.

## Stability Tiers

- Stable: the Public Modules and curated public surface documented in this file.
- Experimental: new curated helpers are experimental until they are added to the public surface snapshots and called out here.
- Internal: implementation helpers and any item not exposed through `contract`, `id_catalog`, `ids`, `metrics`, `prelude`, or `public_api`.

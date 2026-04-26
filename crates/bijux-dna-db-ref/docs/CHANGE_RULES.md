# Change Rules

These rules govern changes to reference database resolution, runtime config
schemas, provider traits, and public reference contracts.

## Breaking Changes

A change is breaking when existing callers, config files, lock files, resolved
contracts, or provenance payloads can observe different behavior for the same
inputs.

Breaking changes include:

- Renaming, removing, or changing public fields on catalog, lock, species,
  reference, panel, map, or provenance records.
- Changing species alias normalization or default-build selection.
- Changing contig normalization semantics.
- Changing lock reference parsing or checksum validation rules.
- Changing panel/map selection defaults.
- Changing `RefService`, `ReferenceProvider`, `PanelProvider`, or `MapProvider`
  method signatures.
- Changing `SpeciesContext` projection semantics.

Breaking changes require docs, tests, and affected downstream caller updates in
the same work item.

## Non-Breaking Changes

The following are normally non-breaking when existing behavior remains stable:

- Adding new checked-in species, build, panel, map, or reference records.
- Adding optional serialized fields with defaults.
- Adding a new stable operation while preserving existing operation behavior.
- Clarifying docs without behavior changes.

## Required Updates

- Managed operations: `docs/COMMANDS.md`.
- Source and test layout: `docs/ARCHITECTURE.md` and boundary tests.
- Boundary and dependency rules: `docs/BOUNDARY.md`, `docs/DEPENDENCIES.md`,
  and dependency boundary tests.
- Public surface: `docs/PUBLIC_API.md` and public API boundary tests.
- Config, lock, and resolver contracts: `docs/CONTRACTS.md`.
- Test ownership: `docs/TESTS.md`.

## Verification

Run the narrowest relevant suite during development and the full crate suite
before handoff:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ref --no-default-features
```

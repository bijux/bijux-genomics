# bijux-dna-testkit

`bijux-dna-testkit` provides shared test-only helpers for the genomics
workspace. It owns deterministic clocks, seeded RNG helpers, fixture readers,
snapshot normalization, temporary test paths, and workspace-aware text loading.

The crate is deliberately not a production dependency owner. It must not contain
domain semantics, product runtime behavior, CLI adapters, process execution, or
network access.

## Public Surface

- `determinism`: fixed clocks, seeded RNG, timestamp-field stripping, and stable
  JSON assertions.
- `fixtures`: fixture text/JSON readers and JSON shape assertions.
- `snapshots`: text/JSON normalization, snapshot names, and locale/timezone
  setup for tests.
- `temp`: isolated temp dirs, contained relative paths, and deterministic
  directory listings.
- `workspace_support`: workspace-root and text-loading helpers for tests.
- `public_api`: curated mirror of the stable root re-export surface.

## Documentation

The crate keeps one root `README.md`. All other crate docs live under `docs/`
and are indexed from [docs/INDEX.md](docs/INDEX.md).

Key docs:

- [docs/COMMANDS.md](docs/COMMANDS.md): SSOT for callable testkit operations.
- [docs/BOUNDARY.md](docs/BOUNDARY.md): ownership and forbidden surfaces.
- [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md): dependency graph rules.
- [docs/PUBLIC_API.md](docs/PUBLIC_API.md): public module and root export contract.
- [docs/SNAPSHOT_POLICY.md](docs/SNAPSHOT_POLICY.md): fixture and snapshot rules.
- [docs/TESTS.md](docs/TESTS.md): local verification commands.

## Verification

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --no-default-features
```

# Index

## Scope

- Deterministic reference, species, panel, and map metadata resolution.

## Effects

- Read-only config and lock loading.
- Pure in-memory contract projection.
- No writes, network access, process spawning, or runtime execution.

## Boundaries

- Depends on `bijux-dna-domain-vcf` for shared VCF species and coverage
  contracts.
- Does not depend on planners, stages, runners, APIs, CLIs, or environments.

## Extension Points

- Add new species/build bundles through checked-in config and lock additions.
- Add provider trait methods only when the public API and contract docs are
  updated.
- Add resolver operations only when `docs/COMMANDS.md` is updated.

## How to Test

- Unit and integration tests are mapped in [TESTS.md](TESTS.md).

## Documents

- [ARCHITECTURE.md](ARCHITECTURE.md)
- [BOUNDARY.md](BOUNDARY.md)
- [CHANGE_RULES.md](CHANGE_RULES.md)
- [COMMANDS.md](COMMANDS.md)
- [CONTRACTS.md](CONTRACTS.md)
- [DEPENDENCIES.md](DEPENDENCIES.md)
- [PUBLIC_API.md](PUBLIC_API.md)
- [SCOPE.md](SCOPE.md)
- [TESTS.md](TESTS.md)

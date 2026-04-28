# bijux-dna-policies

`bijux-dna-policies` enforces repository policy checks for source layout, dependency boundaries, documentation hygiene, ownership, fixtures, snapshots, and deterministic governance.

## Ownership
This crate owns policy assertion macros, guardrail configuration, deterministic repository scanning, and actionable policy diagnostics.

It must not own product execution, CLI routing, domain semantics, runtime orchestration, generated-output mutation, process spawning, network access, or snapshot blessing outside explicit test review.

## Public Surface
Stable root exports from `src/lib.rs`:

- `check`
- `GuardrailConfig`
- `policy_assert!`
- `policy_assert_eq!`
- `policy_assert_ne!`
- `policy_panic!`

Public modules:

- `public_api`
- `policy_diagnostics`

## Documentation
- [docs/INDEX.md](docs/INDEX.md)
- [docs/COMMANDS.md](docs/COMMANDS.md)
- [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md)
- [docs/PUBLIC_API.md](docs/PUBLIC_API.md)
- [docs/TESTS.md](docs/TESTS.md)

## Tests
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --no-default-features
```

## Repository Policy
This crate follows repository governance documentation. `/Users/bijan/bijux/bijux-genomics/README.md`,
`README.md`, and `README.md`; re-read
those files before editing this child repository or making commits.

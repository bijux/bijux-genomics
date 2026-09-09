# bijux-dna Tests

## Test Layout
`tests/` should contain executable integration test entrypoints, owned test modules, snapshots, and
shared path helpers. It should not contain README placeholder files or empty taxonomy directories.

```text
tests/
├── benchmark_readiness.rs
├── benchmark/
│   └── readiness/
├── boundaries.rs
├── boundaries/
├── contracts.rs
├── contracts/
├── guardrails.rs
├── schemas.rs
├── schemas/
├── snapshots.rs
├── snapshots/
└── support/
```

## Coverage Map
- `tests/benchmark_readiness.rs`: active stage/tool scope, adapters, parser fixtures, output declarations/contracts, expected results and report maps. JSON and TSV assertions live in `tests/benchmark/readiness/`. One executable retains individual case identities under report modules; the slow roster matches the unchanged function-name suffixes. The standard harness shares one process environment guard, while nextest can schedule separate cases.
- `tests/boundaries.rs`: layout, docs placement, dependency, public-surface, no-process-spawn, and
  guardrail checks.
- `tests/boundaries/architecture_tree.rs`: root, docs, source, command, and public API tree
  contract, including the integration-test taxonomy and support-helper location.
- `tests/boundaries/command_inventory.rs`: `docs/COMMANDS.md` command inventory, section, alias,
  and verification-command contract.
- `tests/contracts.rs`: command behavior, bank, dry-run, and HPC layout contracts.
- `tests/contracts/dry_run.rs`: dry-run command families and VCF toy execution coverage.
- `tests/guardrails.rs`: shared policy guardrail smoke for this crate.
- `tests/schemas.rs`: public API surface snapshots.
- `tests/snapshots.rs`: CLI help output snapshots and docs-to-help checks.
- `src/process_exit.rs` unit tests: operator error and exit-code behavior.

Markdown files under `tests/` are not allowed; test taxonomy belongs here.

## Commands
- Fast layout/dependency check:

```text
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --test boundaries --no-default-features
```

- Command behavior contracts:

```text
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --test contracts --no-default-features
```

- Full crate:

```text
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna --no-default-features
```

## Snapshot Rules
Update snapshots only when the command surface, output format, or public export intentionally
changes. Snapshot diffs must be reviewed as contract changes, not accepted mechanically.

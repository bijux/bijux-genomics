# Tests

The test tree is organized by the contract each suite protects.

## Suite Map

- `tests/boundaries.rs`: guardrails, purity, dependency graph, docs placement, and architecture
  checks.
- `tests/contracts.rs`: stage specs, registry, command inventory, public API, docs contracts,
  symmetry, and contract snapshots.
- `tests/contracts/observer/`: observer parsing, fixture-bank invariants, and determinism.
- `tests/determinism.rs`: fixture stability checks.
- `tests/semantics/`: behavior checks that exercise parsed reports and metrics.
- `tests/guardrails.rs`: shared repository guardrail policy.

## Important Directories

- `tests/boundaries/`: layout, dependency, docs, and effect-boundary assertions.
- `tests/contracts/observer/`: observer parser coverage.
- `tests/fixtures/`: parser fixtures, stage output bank, and contract snapshots.
- `tests/snapshots/`: schema shape snapshots.

The crate intentionally has no `tests/README.md` files and no placeholder
schema directory. New tests should live under the suite that owns the contract
being protected.

## Verification

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-fastq --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-fastq --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-fastq --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-fastq --no-default-features
```

## Failure Meaning

- Boundary failures mean layout, layering, dependency, docs placement, or effect
  ownership drifted.
- Contract failures mean public plugin behavior, registry coverage, parser
  outputs, snapshots, or docs contracts changed.
- Determinism failures mean fixture canonicalization or snapshot stability changed.
- Semantic failures mean report parsing or required metric coverage changed.

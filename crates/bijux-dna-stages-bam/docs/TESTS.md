# Tests

The test tree is organized by the contract each suite protects.

## Suite Map

- `tests/boundaries.rs`: crate tree, docs placement, dependency graph, purity, and pipeline
  boundary guardrails.
- `tests/contracts.rs`: stage contracts, plugin behavior, registry completeness, observer parser
  contracts, snapshots, and structure limits.
- `tests/determinism.rs`: fixture and canonical JSON determinism.
- `tests/guardrails.rs`: shared repository guardrail policy.
- `tests/semantics.rs`: BAM metric semantic completeness and metric discovery behavior.

## Important Directories

- `tests/boundaries/`: architecture, dependency, docs, and purity rules.
- `tests/contracts/observer/`: parser coverage and snapshot contracts.
- `tests/fixtures/observer/default/`: source parser fixtures.
- `tests/fixtures/observer_snapshot/default/`: aggregate observer snapshot.
- `tests/fixtures/observer_snapshots/default/`: per-parser snapshots.
- `tests/fixtures/stage_contracts/default/`: stage contract snapshot.
- `tests/semantics/metrics/`: metric discovery and completeness assertions.

The crate intentionally has no `tests/README.md` files and no placeholder
schema directory. New tests should live under the suite that owns the contract
being protected.

## Verification

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-bam --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-bam --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-bam --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-bam --test semantics --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stages-bam --no-default-features
```

## Failure Meaning

- Boundary failures mean layout, layering, docs placement, or dependency ownership drifted.
- Contract failures mean public plugin behavior, parser outputs, snapshots, or the stage registry
  changed.
- Determinism failures mean fixture canonicalization or snapshot stability changed.
- Semantic failures mean metric discovery or required metric coverage changed.

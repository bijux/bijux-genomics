# bijux-dna-domain-fastq Tests

The test suite locks domain boundaries, manifest parity, public surface, deterministic fixtures,
semantic behavior, and contract snapshots.

## Test Map

| Surface | Test file or directory | Contract |
| --- | --- | --- |
| Boundaries | `tests/boundaries.rs`, `tests/boundaries/*` | Docs layout, command-free surface, dependency graph, guardrails, purity, and deterministic source shape. |
| Contracts | `tests/contracts.rs`, `tests/contracts/*` | Stage contracts, domain manifest parity, tool manifests, public surface, SSOT literals, and snapshots. |
| Determinism | `tests/determinism.rs`, `tests/determinism/*` | Fixture stability and reproducibility checks. |
| Semantics | `tests/semantics.rs`, `tests/semantics/*` | Retention, canonical params, observability, execution support, tool models, and stage params. |
| Invariants | `tests/semantics/invariants/*` | Invariant specs, metric fixtures, and evaluation behavior. |

The raw FASTQ parser bank for benchmark-facing observer fixtures lives at
`tests/fixtures/bench/parsers/fastq/<stage>/<tool>/` in the repository root.
Domain parser contracts consume that bank directly so raw tool outputs and expected normalized
JSON stay governed in one location.

## Commands

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-fastq --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-fastq --no-default-features --test boundaries
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-fastq --no-default-features --test contracts
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-fastq --no-default-features --test determinism
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-fastq --no-default-features --test semantics
CARGO_TARGET_DIR=artifacts/cargo-target cargo clippy -p bijux-dna-domain-fastq --all-targets --no-default-features -- -D warnings
```

## Artifact Discipline

Rust build products must use `CARGO_TARGET_DIR=artifacts/cargo-target`. Snapshot updates are
governed outputs and should be staged only when the snapshot change is the intended contract update.

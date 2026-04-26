# bijux-dna-domain-vcf Tests

The test suite locks VCF domain contracts, generated registry parity, guardrails, and crate layout.

## Test Map

| Surface | Test file or directory | Contract |
| --- | --- | --- |
| Contracts | `tests/contracts.rs` | Stage taxonomy, transitions, params, metrics, invariants, registry output, and committed config parity. |
| Guardrails | `tests/guardrails.rs` | Repository policy guardrails for the crate. |
| Boundaries | `tests/boundaries.rs`, `tests/boundaries/*` | Docs layout, command-free surface, dependency graph, and source/test tree shape. |

## Commands

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-vcf --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-vcf --no-default-features --test contracts
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-vcf --no-default-features --test guardrails
CARGO_TARGET_DIR=artifacts/cargo-target cargo clippy -p bijux-dna-domain-vcf --all-targets --no-default-features -- -D warnings
```

## Artifact Discipline

Rust build products must use `CARGO_TARGET_DIR=artifacts/cargo-target`. Generated config artifacts
under `configs/ci/` should change only when registry rendering intentionally changes.

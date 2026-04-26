# bijux-dna-domain-vcf Dependencies

The VCF domain crate is upstream of execution and downstream of shared serialization/error
contracts. Its dependency graph must stay small.

## Allowed Production Dependencies

- `anyhow`: error context for invariant, taxonomy, and panel-governance validation.
- `schemars`: JSON schema derivation for public parameter and metric contracts.
- `serde`, `serde_json`: typed serialization for public contract payloads.

## Allowed Test Dependencies

- `bijux-dna-policies`: workspace guardrail checks.

## Forbidden Dependencies

This crate must not depend on API, benchmark, database, developer-control-plane, engine,
environment, planner, runner, runtime, stage execution, infrastructure IO, or command crates.
Those layers may consume VCF domain truth; they must not define it.

## Verification

Use `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-vcf --no-default-features --test boundaries`
to verify the direct dependency boundary.

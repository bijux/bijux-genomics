# bijux-dna-domain-fastq Dependencies

The FASTQ domain crate is upstream of execution. Its dependency graph must point toward shared
domain primitives, serialization, governed parsing, and test infrastructure only.

## Allowed Production Dependencies

- `anyhow`: error context for domain and parser failures.
- `bijux-dna-core`: shared identifiers and primitive domain contracts.
- `bijux-dna-infra`: governed YAML/JSON parsing and filesystem helpers.
- `flate2`: gzip-aware FASTQ discovery and report helpers.
- `schemars`, `serde`, `serde_json`: typed contract schemas and canonical JSON.
- `sha2`: deterministic hashing for contracts and provenance.
- `tracing`: diagnostic events without runtime ownership.
- `uuid`: typed identifiers used in public domain surfaces.

## Allowed Test Dependencies

- `bijux-dna-policies`: workspace guardrails.
- `bijux-dna-testkit`: shared fixture and snapshot helpers.
- `insta`: contract snapshots.
- `walkdir`: deterministic fixture and source tree walks.

## Forbidden Dependencies

This crate must not depend on API, benchmark runner, database, developer-control-plane, engine,
environment, planner, runner, runtime, stage execution, or command crates. Those layers consume
FASTQ domain truth; they do not define it.

## Verification

Use `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-fastq --no-default-features --test boundaries`
to verify the direct dependency boundary.

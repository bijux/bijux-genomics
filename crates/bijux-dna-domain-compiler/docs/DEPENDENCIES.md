# bijux-dna-domain-compiler Dependencies

This crate sits between authored domain metadata and generated configuration files. Its dependency
graph must stay pointed at schema/domain truth and repository IO helpers, not at execution layers.

## Allowed production dependencies

- `anyhow`: error context for validation and generation failures.
- `clap`: binary argument parsing for the two crate-owned commands.
- `serde`, `serde_json`, `toml`: parsing and rendering authored/generated config structures.
- `sha2`: deterministic source-content hash generation.
- `bijux-dna-infra`: repository filesystem helpers and YAML parsing.
- `bijux-dna-domain-fastq`, `bijux-dna-domain-bam`: canonical domain vocabulary contracts used by
  compiler validation.

## Allowed test dependencies

- `bijux-dna-policies`: workspace guardrail checks.
- `tempfile`: isolated generated-output directories under the repository `artifacts/` tree.

## Forbidden dependencies

The compiler must not depend on runtime, runner, engine, planner, stage execution, API, benchmark,
database, or developer-control-plane crates. Those layers may consume generated compiler outputs,
but the compiler must remain upstream of execution.

Internal `bijux-dna-*` dependencies must be declared through the workspace catalog. Direct
`path = "../..."` declarations bypass the shared dependency graph and are rejected by the boundary
tests.

## Verification

Use `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-domain-compiler --no-default-features --test boundaries`
to verify the direct dependency boundary.

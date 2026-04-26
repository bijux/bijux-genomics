# bijux-dna-environment Dependencies

The environment crate sits between shared contracts and runtime consumers. Its dependency graph must
stay small and must not point at higher orchestration layers.

## Allowed Production Dependencies

- `anyhow`: caller-facing context for shell and smoke helper errors.
- `bijux-dna-core`: canonical contract helpers used by schema tests and public contracts.
- `bijux-dna-infra`: repository config lookup, atomic file operations, directory creation, and TOML
  parsing with the `yaml` feature already used elsewhere in the crate family.
- `bijux-dna-runtime`: shared runtime model dependency for cross-crate compatibility.
- `regex`: Dockerfile version ARG parsing.
- `serde`, `serde_json`: serialized environment model contracts.
- `sha2`: reference content digests.
- `thiserror`: typed `EnvError`.
- `tracing`: instrumentation compatibility for environment consumers.
- `uuid`: shared identity support for downstream environment records.

## Allowed Test Dependencies

- `bijux-dna-policies`: workspace guardrails.
- `bijux-dna-testkit`: fixture, stable JSON, and artifact-aware temp helpers.
- `tempfile`: temporary directories when routed through testkit or explicit artifact roots.
- `walkdir`: boundary source scans.

## Forbidden Dependencies

This crate must not depend on API, CLI, developer-control-plane, domain, planner, pipeline, runner,
stage, benchmark, analysis, database, science, or environment QA crates. Those layers may call this
crate; they must not become prerequisites for compiling it.

## Verification

The dependency boundary is locked by:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment --no-default-features --test boundaries
```

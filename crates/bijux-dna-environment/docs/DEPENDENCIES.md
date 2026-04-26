# bijux-dna-environment Dependencies

The environment crate sits between shared contracts and runtime consumers. Its dependency graph must
stay small and must not point at higher orchestration layers.

## Allowed Production Dependencies

- `anyhow`: caller-facing context for shell and smoke helper errors.
- `bijux-dna-infra`: repository config lookup, atomic file operations, directory creation, and TOML
  parsing with the `yaml` feature already used elsewhere in the crate family.
- `regex`: Dockerfile version ARG parsing.
- `serde`: serialized environment model contracts.
- `sha2`: reference content digests.
- `thiserror`: typed `EnvError`.

## Allowed Test Dependencies

- `bijux-dna-core`: canonical JSON helpers used by schema snapshots.
- `bijux-dna-policies`: workspace guardrails.
- `bijux-dna-testkit`: fixture, stable JSON, and artifact-aware temp helpers.
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

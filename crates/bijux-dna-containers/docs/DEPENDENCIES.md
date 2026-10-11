# bijux-dna-containers Dependencies

The environment crate sits between shared contracts and runtime consumers. Its dependency graph must
stay small and must not point at higher orchestration layers.

## Allowed Production Dependencies

- `anyhow`: caller-facing context for shell and smoke helper errors.
- `bijux-dna-infra`: repository config lookup, atomic file operations, directory creation, and TOML
  parsing with the `yaml` feature already used elsewhere in the crate family.
- `regex`: Dockerfile version parsing and contract-owned smoke admission.
- `schemars`: JSON Schema generation for evidence and embedded image contracts.
- `serde`: serialized environment model contracts.
- `serde_json`: typed evidence loading, label inspection, and schema rendering.
- `sha2`: reference content digests.
- `thiserror`: typed `EnvError`.
- `toml`: strict loading of the canonical container catalog during build preparation.

## Allowed Test Dependencies

- `bijux-dna-core`: canonical JSON helpers used by schema snapshots.
- `bijux-dna-policies`: workspace guardrails.
- `bijux-dna-testkit`: fixture, stable JSON, and artifact-aware temp helpers.
- `tempfile`: lifecycle ownership for contract-build test artifacts.
- `walkdir`: boundary source scans.

## Forbidden Dependencies

This crate must not depend on API, CLI, developer-control-plane, domain, planner, pipeline, runner,
stage, benchmark, analysis, database, science, or environment QA crates. Those layers may call this
crate; they must not become prerequisites for compiling it.

## Verification

The dependency boundary is locked by:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers --no-default-features --test boundaries
```

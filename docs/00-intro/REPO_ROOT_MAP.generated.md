<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: scripts/tooling/generate-repo-root-map.sh -->

# REPO_ROOT_MAP

## Purpose
Generated map of repository root entries with inferred ownership and intent.

## Scope
Top-level workspace paths only.

## Non-goals
- Replacing detailed per-subtree architecture docs.

## Contracts
- Ownership for config paths is sourced from `configs/OWNERS.toml`.
- Script subtree intent is sourced from README `Purpose:` lines validated by `scripts/checks/tree-intent.sh`.

| Path | Kind | Owner | Purpose |
|---|---|---|---|
| `Cargo.lock` | `file` | `-` | - |
| `Cargo.toml` | `file` | `-` | - |
| `LICENSE` | `file` | `-` | - |
| `Makefile` | `file` | `-` | - |
| `README.md` | `file` | `-` | - |
| `artifacts` | `dir` | `-` | - |
| `assets` | `dir` | `-` | - |
| `audit-allowlist.toml` | `file` | `-` | - |
| `bin` | `dir` | `-` | runtime boundary helpers for strict isolated execution. |
| `configs` | `dir` | `-` | - |
| `containers` | `dir` | `-` | - |
| `crates` | `dir` | `-` | - |
| `deny.toml` | `file` | `-` | - |
| `docs` | `dir` | `-` | - |
| `domain` | `dir` | `-` | - |
| `examples` | `dir` | `-` | - |
| `makefiles` | `dir` | `-` | - |
| `mkdocs.yml` | `file` | `-` | - |
| `rust-toolchain.toml` | `file` | `-` | - |
| `scripts` | `dir` | `-` | strict index of supported script areas and allowed usage. |
| `target` | `dir` | `-` | - |

## Script Intent
| Script Path | Purpose |
|---|---|
| `scripts/_lib` | shared shell helper library for supported scripts. |
| `scripts/assets` | deterministic asset and golden fixture refresh scripts. |
| `scripts/checks` | enforce CI and repository script policies. |
| `scripts/containers` | container runtime build/lint/smoke entrypoints. |
| `scripts/docs` | docs checks and documentation tooling entrypoints. |
| `scripts/domain` | domain schema validation and drift detection. |
| `scripts/examples` | generate, validate, and run curated repository examples. |
| `scripts/experimental` | quarantined non-supported scripts not called from make/CI. |
| `scripts/hpc` | HPC-specific operational scripts. |
| `scripts/lab` | manual lab runs and benchmark orchestration. |
| `scripts/smoke` | unified local smoke entrypoint and domain-specific smoke commands. |
| `scripts/test` | test triage and deterministic toy-run wrappers. |
| `scripts/tooling` | repository tooling wrappers and inventories. |

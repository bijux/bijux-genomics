<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dev-dna -- tooling run generate-repo-root-map -->

# REPO_ROOT_MAP

## Purpose
Generated map of repository root entries with inferred ownership and intent.

## Scope
Top-level workspace paths only.

## Non-goals
- Replacing detailed per-subtree architecture docs.

## Contracts
- Ownership for config paths is sourced from `configs/OWNERS.toml`.
- Script subtree intent is sourced from README `Purpose:` lines.

| Path | Kind | Owner | Purpose |
|---|---|---|---|
| `Cargo.toml` | `file` | `-` | - |
| `artifacts` | `dir` | `-` | - |
| `crates` | `dir` | `-` | - |
| `mkdocs.yml` | `file` | `-` | - |
| `LICENSE` | `file` | `-` | - |
| `Makefile` | `file` | `-` | - |
| `Cargo.lock` | `file` | `-` | - |
| `docs` | `dir` | `-` | - |
| `README.md` | `file` | `-` | - |
| `audit-allowlist.toml` | `file` | `-` | - |
| `examples` | `dir` | `-` | - |
| `makes` | `dir` | `-` | - |
| `configs` | `dir` | `-` | - |
| `scripts` | `dir` | `-` | strict index of supported script areas and allowed usage. |
| `containers` | `dir` | `-` | - |
| `rust-toolchain.toml` | `file` | `-` | - |
| `assets` | `dir` | `-` | - |
| `domain` | `dir` | `-` | - |

## Script Intent
| Script Path | Purpose |
|---|---|
| `scripts/experimental` | quarantined non-supported scripts not called from make/CI. |
| `scripts/tooling` | repository tooling wrappers and inventories. |
| `scripts/_lib` | shared shell helper library for supported scripts. |

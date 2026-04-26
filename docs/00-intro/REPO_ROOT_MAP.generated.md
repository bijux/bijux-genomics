<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-repo-root-map -->

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
| `rustfmt.toml` | `file` | `-` | - |
| `LICENSE` | `file` | `-` | - |
| `Makefile` | `file` | `-` | - |
| `science` | `dir` | `-` | - |
| `Cargo.lock` | `file` | `-` | - |
| `docs` | `dir` | `-` | - |
| `NOTICE` | `file` | `-` | - |
| `README.md` | `file` | `-` | - |
| `audit-allowlist.toml` | `file` | `-` | - |
| `examples` | `dir` | `-` | - |
| `makes` | `dir` | `-` | - |
| `configs` | `dir` | `-` | - |
| `containers` | `dir` | `-` | - |
| `rust-toolchain.toml` | `file` | `-` | - |
| `assets` | `dir` | `-` | - |
| `domain` | `dir` | `-` | - |

## Automation Intent
| Control Plane Path | Purpose |
|---|---|
| `crates/bijux-dna-dev` | - |
| `makes` | - |

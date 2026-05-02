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
| `Cargo.lock` | `file` | `-` | - |
| `Cargo.toml` | `file` | `-` | - |
| `LICENSE` | `file` | `-` | - |
| `Makefile` | `file` | `-` | - |
| `NOTICE` | `file` | `-` | - |
| `README.md` | `file` | `-` | - |
| `artifacts` | `dir` | `-` | - |
| `assets` | `dir` | `-` | - |
| `audit-allowlist.toml` | `file` | `-` | - |
| `configs` | `dir` | `-` | - |
| `containers` | `dir` | `-` | - |
| `crates` | `dir` | `-` | - |
| `docs` | `dir` | `-` | - |
| `domain` | `dir` | `-` | - |
| `examples` | `dir` | `-` | - |
| `makes` | `dir` | `-` | - |
| `mkdocs.yml` | `file` | `-` | - |
| `rust-toolchain.toml` | `file` | `-` | - |
| `rustfmt.toml` | `file` | `-` | - |
| `science` | `dir` | `-` | - |
| `target` | `dir` | `-` | - |

## Automation Intent
| Control Plane Path | Purpose |
|---|---|
| `crates/bijux-dna-dev` | - |
| `makes` | - |

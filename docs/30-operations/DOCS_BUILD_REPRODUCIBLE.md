# DOCS_BUILD_REPRODUCIBLE

## Purpose
Define exact reproducible docs build steps and pinned dependency/config locations.

## Scope
Applies to docs builds executed via Make and `cargo run -q -p bijux-dna-dev -- tooling run docs-build`.

## Non-goals
- Replacing lower-level MkDocs theme/content guidance.

## Contracts
- Python dependencies are pinned in
  [configs/docs/requirements.txt](../../configs/docs/requirements.txt).
- Build behavior is pinned in
  [configs/docs/mkdocs.toml](../../configs/docs/mkdocs.toml).
- Docs commands are executed through `cargo run -q -p bijux-dna-dev -- tooling run setup-docs-venv` and `cargo run -q -p bijux-dna-dev -- tooling run docs-build`.

## Reproducible Steps
```bash
cargo run -q -p bijux-dna-dev -- tooling run setup-docs-venv
DOCS_VENV=artifacts/docs/.venv DOCS_CFG=configs/docs/mkdocs.toml cargo run -q -p bijux-dna-dev -- tooling run docs-build build
DOCS_VENV=artifacts/docs/.venv DOCS_CFG=configs/docs/mkdocs.toml cargo run -q -p bijux-dna-dev -- tooling run docs-build lint
```

Expected output:
- Site output at `artifacts/docs/site`
- Behavior determined by [configs/docs/mkdocs.toml](../../configs/docs/mkdocs.toml)

## Pinned Inputs
- [configs/docs/requirements.txt](../../configs/docs/requirements.txt)
- [configs/docs/mkdocs.toml](../../configs/docs/mkdocs.toml)
- [mkdocs.yml](../../mkdocs.yml)

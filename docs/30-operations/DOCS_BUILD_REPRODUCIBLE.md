# DOCS_BUILD_REPRODUCIBLE

## Purpose
Define exact reproducible docs build steps and pinned dependency/config locations.

## Scope
Applies to docs builds executed via Make and `scripts/tooling/docs-build.sh`.

## Non-goals
- Replacing lower-level MkDocs theme/content guidance.

## Contracts
- Python dependencies are pinned in `configs/docs/requirements.txt`.
- Build behavior is pinned in `configs/docs/mkdocs.toml`.
- Docs commands are executed through `scripts/tooling/setup-docs-venv.sh` and `scripts/tooling/docs-build.sh`.

## Reproducible Steps
```bash
./scripts/run.sh tooling setup-docs-venv
DOCS_VENV=artifacts/docs/.venv DOCS_CFG=configs/docs/mkdocs.toml ./scripts/run.sh tooling docs-build build
DOCS_VENV=artifacts/docs/.venv DOCS_CFG=configs/docs/mkdocs.toml ./scripts/run.sh tooling docs-build lint
```

Expected output:
- Site output at `artifacts/docs/site`
- Behavior determined by `configs/docs/mkdocs.toml`

## Pinned Inputs
- `configs/docs/requirements.txt`
- `configs/docs/mkdocs.toml`
- `mkdocs.yml`

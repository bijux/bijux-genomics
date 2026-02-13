# MkDocs Build

## Purpose
Define exact MkDocs build commands and dependency locations.

## Scope
Applies to root documentation builds executed via Make and CI.

## Non-goals
- Covering crate-local docs generation workflows.

## Contracts
- Dependencies are sourced from `configs/docs/requirements.txt`.
- Environment setup is performed only through `scripts/tooling/setup-docs-venv.sh`.
- CI/docs gates must use `make docs-lint` behavior (`mkdocs build --strict`).

## Commands
```bash
make docs            # non-strict local build
make docs-lint       # strict local build
make docs-isolate    # strict build and checks under isolate
```

Equivalent internal commands:
```bash
DOCS_REQ=configs/docs/requirements.txt ./scripts/run.sh tooling setup-docs-venv
. artifacts/docs/.venv/bin/activate
mkdocs build --strict --site-dir artifacts/docs/site
```

## Dependency Locations
- Requirements file: `configs/docs/requirements.txt`
- Venv/bootstrap wrapper: `scripts/tooling/setup-docs-venv.sh`
- Docs checks entrypoints: `scripts/docs/*.sh`

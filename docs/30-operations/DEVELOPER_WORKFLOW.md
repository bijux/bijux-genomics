# Developer Workflow

## What
Exact local commands that mirror CI.

## Why
Ensures local checks match pipeline enforcement.

## Non-goals
- Performance tuning.

## Contracts
CI enforces these commands via `make` and policy tests.

## Examples
```bash
make fmt
make lint
make test
make policy-fast
make policy-full
```

Outputs:
- test artifacts under `target/`
- docs build under `site/` (mkdocs)

## Failure modes
Any failure must be resolved locally before merging.

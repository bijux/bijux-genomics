# Developer Workflow

## What
Exact local commands that mirror CI.

## Why
Ensures local checks match pipeline enforcement.

## Non-goals
- Performance tuning.

## Contracts
CI enforces these commands via `make` and policy tests.
- Any command that touches Cargo must use the shared `artifacts/` environment.

## Examples
```bash
make fmt
make lint
make audit
make test
make coverage
make ci
```

Outputs:
- shared build cache under `artifacts/target/`
- docs build under `artifacts/docs/site/`

## Failure modes
Any failure must be resolved locally before merging.

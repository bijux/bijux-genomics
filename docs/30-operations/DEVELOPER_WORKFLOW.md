# Developer Workflow

## What
Exact local commands that mirror CI.

## Why
Ensures local checks match pipeline enforcement.

## Non-goals
- Performance tuning.

## Contracts
CI enforces these commands via `make` and policy tests.
- Any command that touches Cargo must run under `./bin/isolate`.

## Examples
```bash
./bin/isolate make fmt
./bin/isolate make lint
./bin/isolate make test
./bin/isolate make policy-fast
./bin/isolate make policy-full
```

Outputs:
- isolate-scoped test/build artifacts under `artifacts/isolates/<ISO_TAG>/`
- docs build under `artifacts/isolates/<ISO_TAG>/docs/site/`

## Failure modes
Any failure must be resolved locally before merging.

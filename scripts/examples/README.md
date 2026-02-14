# scripts/examples

Purpose: generate, validate, and run curated repository examples.

Back to index: `scripts/README.md`.

Commands:
- `scripts/examples/generate-index.sh`
- `scripts/examples/check-index.sh`
- `scripts/examples/check-drift.sh <example-id>` (manual, non-CI by default)
- `scripts/examples/run.sh <example-id>`

`run.sh` writes a `bundle.tar.gz` file under `artifacts/examples/` containing:
- `manifest.json`
- `metrics.json`
- `logs.txt`
- `plan.json`
- `explain.json`
- `report.json`
- `run_report.json`

Requires: bash, python3, rg, coreutils.
Exit codes: 0 success; 1 policy/validation failure; 2 usage/config error; 127 missing dependency.

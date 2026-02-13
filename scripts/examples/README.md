# scripts/examples

Purpose: generate, validate, and run curated repository examples.

Back to index: `scripts/README.md`.

Commands:
- `scripts/examples/generate-index.sh`
- `scripts/examples/check-index.sh`
- `scripts/examples/check-drift.sh <example-id>`
- `scripts/examples/run.sh <example-id>`

Requires: bash, python3, rg, coreutils.
Exit codes: 0 success; 1 policy/validation failure; 2 usage/config error; 127 missing dependency.

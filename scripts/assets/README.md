# Assets Scripts

## What
Deterministic refresh entrypoints for toy datasets and golden fixtures.

## Entry Points
- `scripts/assets/refresh-toy.sh`: regenerate `assets/toy/core-v1` from deterministic literals.
- `scripts/assets/refresh-golden.sh`: regenerate `assets/golden/toy-runs-v1` via `scripts/test/toy_runs.py`.

## Rules
- Stage outputs under `artifacts/tmp/` first, then copy to `assets/`.
- No interactive prompts.
- Scripts must stay deterministic and idempotent.

# CI

## What
CI enforces formatting, lint, tests, policies, and docs build for the bijux-dna workspace.

## Why
Keeps code and docs in sync.

## Non-goals
- Performance optimization.

## Contracts
CI runs:
- `make fmt`
- `make lint`
- `make test`
- `make policy-full`
- `mkdocs build --strict`

## Examples
Run locally with the same commands before pushing.

## Failure modes
Any failure blocks merge.

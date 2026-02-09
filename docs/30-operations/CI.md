# CI

## What
CI enforces the minimal deterministic gate for the bijux-dna workspace.

## Why
Keeps code and docs in sync.

## Non-goals
- Performance optimization.

## Contracts
CI runs:
- `make ci`

`make ci` is exactly:
- `make fmt`
- `make lint`
- `make audit`
- `make coverage`

`make check` is the same minimal gate as `make ci`.

## Examples
Run locally with the same commands before pushing.

## Failure modes
Any failure blocks merge.

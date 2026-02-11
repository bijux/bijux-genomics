# CI

## What
CI enforces the minimal deterministic gate for the bijux-dna workspace.

## Why
Keeps code and docs in sync.

## Non-goals
- Performance optimization.

## Contracts
CI runs only isolated commands.
- `./bin/isolate make ci`

`make ci` executes isolated gates (`fmt-isolate`, `lint-isolate`, `audit-isolate`, `test-isolate`, `docs-isolate`).

`make check` should be run through isolate as well: `./bin/isolate make check`.

## Examples
Run locally with the same commands before pushing.

## Failure modes
Any failure blocks merge.

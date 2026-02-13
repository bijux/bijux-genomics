# CI

## What
CI enforces the minimal deterministic gate for the workspace.

## Command
- `./bin/isolate make ci`

## Current `make ci` Gates
- `fmt`
- `lint`
- `audit`
- `test`
- `coverage`

## Isolation Contract
- See `docs/30-operations/ISOLATION.md`.

## Non-goals
- Documenting non-`make ci` target suites.

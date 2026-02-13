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

## Scope
Applies only to the files and workflows referenced in this document.

## Contracts
- Content here is normative where explicitly stated.


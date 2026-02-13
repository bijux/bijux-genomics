# CI

## What
CI enforces the minimal deterministic gate for the workspace.

## Purpose
Define the canonical CI gate contract and isolate invocation for the repository.

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

## HPC Forward-compat
- With HPC enabled, `make ci` still enforces the same gate order and policy checks.
- Path roots and container storage may resolve to HPC profile locations, not local defaults.
- Use profile-aware commands and avoid hardcoded local paths in scripts/docs.

## Non-goals
- Documenting non-`make ci` target suites.

## Scope
Applies only to the files and workflows referenced in this document.

## Contracts
- Content here is normative where explicitly stated.

## Examples
- Local: `./bin/isolate make ci`
- HPC profile enabled: `./bin/isolate --tag ci-hpc make ci` (same gates, different storage roots)

## Failure modes
- Running CI-related scripts without isolation fails by contract.
- HPC path assumptions in docs/scripts can cause false failures when profile roots differ.

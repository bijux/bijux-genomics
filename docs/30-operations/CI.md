# CI

## What
Continuous integration checks for Bijux DNA.

## Why
Prevents regressions in contracts and policies.

## Non-goals
- Full performance benchmarks.

## Contracts
- mkdocs build must pass in strict mode.
- policy tests must be green.

## Examples
- `make lint` and `make docs-lint` in CI.

## Failure modes
- Broken doc links fail docs-lint.

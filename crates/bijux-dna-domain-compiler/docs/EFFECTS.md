## What
Enumerates allowed effects for this crate.

## Why
Maintains deterministic SSOT compilation behavior.

## Non-goals
Runtime side effects unrelated to generated config outputs.

## Contracts
Only generated configs are written; writes are deterministic and reproducible.

## Examples
Generate configs in CI and fail on drift.

## Failure modes
Unexpected filesystem writes or nondeterministic output ordering.

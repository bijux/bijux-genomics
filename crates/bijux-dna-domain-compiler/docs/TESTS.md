## What
Lists primary test surfaces for domain compilation and validation.

## Why
Ensures generated configs remain deterministic and schema-complete.

## Non-goals
Runtime or benchmark correctness testing.

## Contracts
Tests must catch schema gaps, drift, and deterministic output regressions.

## Examples
- `cargo test -p bijux-dna-domain-compiler`
- `cargo test -p bijux-dna-domain-compiler --test guardrails`

## Failure modes
Missing generated headers, stale config output, or invalid domain references.

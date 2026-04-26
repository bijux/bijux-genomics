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
- `cargo test -p bijux-dna-domain-compiler --test determinism_generated_outputs`
- `cargo test -p bijux-dna-domain-compiler --test planned_tool_registry_boundaries`
- `cargo clippy -p bijux-dna-domain-compiler --all-targets -- -D warnings`

## Failure modes
Missing generated headers, stale config output, or invalid domain references.

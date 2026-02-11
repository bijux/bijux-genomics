# Tests

## What
Lists core test entrypoints for bijux-dna-stages-vcf.

## Why
Ensures contract, boundary, and regression coverage is visible.

## Non-goals
- Full workspace orchestration instructions.

## Contracts
- tests/contracts.rs
- tests/guardrails.rs

## Examples
- cargo test -p bijux-dna-stages-vcf --test contracts
- cargo test -p bijux-dna-stages-vcf --test guardrails

## Failure modes
Missing tests or stale fixtures fail policy checks.

# Anti-patterns

## What
Explicit violations that are banned by policy.

## Why
Prevent regressions and keep contracts enforceable.

## Non-goals
- Exhaustive list of all coding style rules.

## Contracts
Policy ownership is reviewed against the [policy crate architecture](../../crates/bijux-dna-policies/docs/ARCHITECTURE.md).
Filenames alone do not prove that policy behavior is duplicated.

Enforced by policy tests:
- [../../crates/bijux-dna-policies/tests/boundaries/surface/policy/id_literal_policy.rs](https://github.com/bijux/bijux-genomics/blob/main/crates/bijux-dna-policies/tests/boundaries/surface/policy/id_literal_policy.rs)
- [../../crates/bijux-dna-policies/tests/boundaries/surface/structure_guards/no_serde_json_writer.rs](https://github.com/bijux/bijux-genomics/blob/main/crates/bijux-dna-policies/tests/boundaries/surface/structure_guards/no_serde_json_writer.rs)
- [../../crates/bijux-dna-policies/tests/boundaries/surface/purity/domain_purity.rs](https://github.com/bijux/bijux-genomics/blob/main/crates/bijux-dna-policies/tests/boundaries/surface/purity/domain_purity.rs)

## Examples
- Duplicated policy decisions outside their owner require source ownership review.
- String IDs in public contracts.
- Direct `serde_json::to_writer` for contract artifacts.
- Domain crates invoking execution effects.

## Failure modes
ID, contract-writing, and domain-effect violations fail CI policy checks with explicit fix guidance.
Policy duplication requires source ownership review.

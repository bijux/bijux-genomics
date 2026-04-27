# Anti-patterns

## What
Explicit violations that are banned by policy.

## Why
Prevent regressions and keep contracts enforceable.

## Non-goals
- Exhaustive list of all coding style rules.

## Contracts
Enforced by policy tests:
- [../../crates/bijux-dna-policies/tests/boundaries/surface/structure_guards/no_policy_duplication.rs](../../crates/bijux-dna-policies/tests/boundaries/surface/structure_guards/no_policy_duplication.rs)
- [../../crates/bijux-dna-policies/tests/boundaries/surface/policy/id_literal_policy.rs](../../crates/bijux-dna-policies/tests/boundaries/surface/policy/id_literal_policy.rs)
- [../../crates/bijux-dna-policies/tests/boundaries/surface/structure_guards/no_serde_json_writer.rs](../../crates/bijux-dna-policies/tests/boundaries/surface/structure_guards/no_serde_json_writer.rs)
- [../../crates/bijux-dna-policies/tests/boundaries/surface/purity/domain_purity.rs](../../crates/bijux-dna-policies/tests/boundaries/surface/purity/domain_purity.rs)

## Examples
- Policy duplication outside `bijux-dna-policies`.
- String IDs in public contracts.
- Direct `serde_json::to_writer` for contract artifacts.
- Domain crates invoking execution effects.

## Failure modes
Violations fail CI policy checks with explicit fix guidance.

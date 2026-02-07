# Anti-patterns

## What
Explicit violations that are banned by policy.

## Why
Prevent regressions and keep contracts enforceable.

## Non-goals
- Exhaustive list of all coding style rules.

## Contracts
Enforced by policy tests:
- `crates/bijux-policies/tests/surface/no_policy_duplication.rs`
- `crates/bijux-policies/tests/surface/id_literal_policy.rs`
- `crates/bijux-policies/tests/surface/no_serde_json_writer.rs`
- `crates/bijux-policies/tests/surface/domain_purity.rs`

## Examples
- Policy duplication outside `bijux-policies`.
- String IDs in public contracts.
- Direct `serde_json::to_writer` for contract artifacts.
- Domain crates invoking execution effects.

## Failure modes
Violations fail CI policy checks with explicit fix guidance.

# Policy Matrix

## What
A mapping of policy tests to their intent.

## Why
Provides a single index for governance checks.

## Non-goals
- Full policy implementation detail.

## Contracts
- Policy IDs are cataloged in [POLICY_INDEX.md](POLICY_INDEX.md).
- Policy tests live under [crates/bijux-dna-policies/tests/](../../crates/bijux-dna-policies/tests/).

## Examples
- docs_required_policy.rs → enforces docs placement.

## Failure modes
- Missing entries lead to governance drift.

## Style
- docs_required_policy.rs
- no_thin_modules_policy.rs
- no_helpers_policy.rs
- mod_naming_policy.rs

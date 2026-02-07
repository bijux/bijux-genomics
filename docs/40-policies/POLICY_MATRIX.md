# Policy Matrix

## What
A mapping of policy tests to their intent.

## Why
Provides a single index for governance checks.

## Non-goals
- Full policy implementation detail.

## Contracts
- Policy tests live in bijux-policies.

## Examples
- docs_required_policy.rs → enforces docs placement.

## Failure modes
- Missing entries lead to governance drift.

## Style
- docs_required_policy.rs
- no_thin_modules_policy.rs
- no_helpers_policy.rs
- no_src_crowd_policy.rs
- mod_naming_policy.rs

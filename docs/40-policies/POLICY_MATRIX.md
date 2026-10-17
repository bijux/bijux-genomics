# Policy Matrix

## What
A mapping of policy tests to their intent.

## Why
Provides a single index for governance checks.

## Non-goals
- Full policy implementation detail.

## Contracts
- Policy IDs are cataloged in [POLICY_INDEX.md](POLICY_INDEX.md).
- Policy tests live under [crates/bijux-dna-policies/tests/](https://github.com/bijux/bijux-genomics/tree/main/crates/bijux-dna-policies/tests).

## Examples
- docs_required_policy.rs → enforces docs placement.

## Failure modes
- Missing entries lead to governance drift.

## Style

- docs_required_policy.rs
- no_empty_dirs_policy.rs
- no_helpers_policy.rs
- mod_naming_policy.rs

Dependency and effect boundaries establish module ownership. The namespace integrity
check rejects empty directories. A compact namespace containing only `mod.rs` must
delegate to existing module files rather than reserve names without implementation.
A fixture directory containing only `.keep` must explain why the directory is retained.
Public-item counts and directory depth are not ownership criteria.

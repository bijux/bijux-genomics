# bijux-dna-benchmark-model Boundary Contract

## Why this crate exists
Defines a focused layer in the bijux-dna architecture with explicit boundaries.

## Allowed dependencies
- Workspace crates required for this layer only.
- No reverse-layer coupling (enforced by policy tests).

## Allowed effects
- Pure data/model crates: no runtime side effects.
- Runtime/CLI/runner crates: controlled filesystem/process/network effects only.

## Notes
Boundary invariants are enforced by bijux-dna-policies contract tests.

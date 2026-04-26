# bijux-dna-domain-bam Boundary Contract

## Why this crate exists
Defines a focused layer in the bijux-dna architecture with explicit boundaries.

## Allowed dependencies
- `bijux-dna-core` for shared IDs and canonical hashing only.
- No reverse-layer coupling (enforced by policy tests).

## Allowed effects
- Pure data/model crates: no runtime side effects.
- No filesystem, process, network, runtime, CLI, runner, or container effects.
- Tests may read crate-local fixtures and snapshots only.

## Notes
Boundary invariants are enforced by bijux-dna-policies contract tests.

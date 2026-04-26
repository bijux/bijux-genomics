# bijux-dna-domain-compiler Boundary Contract

## Why this crate exists
Defines a focused layer in the bijux-dna architecture with explicit boundaries.

## Allowed dependencies
- `bijux-dna-infra` for YAML, filesystem, and generated file helpers.
- Domain crates consumed as canonical catalog sources.
- No reverse-layer coupling (enforced by policy tests).

## Allowed effects
- Read authored `domain/**`, `assets/reference/**`, and container definition metadata.
- Write generated config views under the caller-provided configs directory.
- No pipeline execution, tool execution, runtime orchestration, or network access.

## Notes
Boundary invariants are enforced by bijux-dna-policies contract tests.

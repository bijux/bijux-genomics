# Benchmark execution artifacts

## Ownership

This crate owns admission and publication of immutable benchmark execution evidence
at stable output addresses. It is effectful and uses `bijux-dna-infra` for atomic
file publication and retirement of explicitly owned obsolete stable files.

## Boundaries

This crate does not own command parsing, registry acquisition or native backend implementation.

Scientific workflows own report semantics. Callers supply an explicit ownership
schema and report identity; persisted stage-result validation belongs to
`bijux-dna-bench-model`. No scientific stage, CLI, API or benchmark umbrella is a
dependency of this publication owner.

## Public API

[Publication](docs/PUBLIC_API.md) admits a complete plan before replacing files.

## Architecture

[Architecture](docs/ARCHITECTURE.md) separates schema/owner admission, safe paths,
prior inventories, typed stage-result projection and filesystem writes.

## Tests

[Verification](docs/TESTS.md) covers unsafe identities, immutable native bytes,
foreign inventory refusal, unknown files and escaping public-path projections.

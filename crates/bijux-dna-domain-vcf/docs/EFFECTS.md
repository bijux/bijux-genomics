# bijux-dna-domain-vcf Effects

The crate should be effect-free in production code.

## Allowed Effects

- Deterministic computation over typed inputs.
- Serialization and deserialization of owned parameter and metric payloads.
- String rendering for generated TOML content.
- Test-only reads of committed config artifacts.

## Forbidden Effects

- Process spawning or shelling out.
- Network access.
- Container execution.
- Filesystem writes.
- Runtime state mutation.
- Planner or runner orchestration.

## Determinism

Public catalogs, registry TOML output, invariant decisions, and taxonomy ordering must be stable for
identical inputs.

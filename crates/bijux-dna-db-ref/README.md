# bijux-dna-db-ref

Species/build reference governance for VCF planning.

## Public API
- `resolve_species_context(species, build)`
- `resolve_reference_bundle(species, build)`

## Responsibilities
- Resolve canonical `SpeciesContext` for `{species, build}`.
- Resolve canonical reference bundles and lock metadata.
- Enforce contig normalization policy contracts.
- Provide reference provenance payloads for plan/report artifacts.

## Testing
- Guardrails: `tests/guardrails.rs`
- Documentation and contract fixtures: `tests/*`

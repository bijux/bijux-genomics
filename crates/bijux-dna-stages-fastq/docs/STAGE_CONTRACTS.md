# STAGE_CONTRACTS

## Symmetry
Symmetry is enforced at the contract level (observable inputs/outputs), not file naming.

## Coverage surfaces
`contract_stage_ids()` covers the published FASTQ stage contracts.
`implemented_stages()` covers only the closed execution subset.
`observer_stage_ids()` is the narrower observer-specialized subset documented in `OBSERVERS.md`.

## Registry completeness
`tests/contracts/registry_completeness.rs` ensures every domain stage appears in the stage registry.
When adding a stage, update the registry and this document.

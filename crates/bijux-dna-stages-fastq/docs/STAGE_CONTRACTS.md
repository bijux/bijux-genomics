# STAGE_CONTRACTS

## Symmetry
Symmetry is enforced at the contract level (observable inputs/outputs), not file naming.

## Coverage surfaces
`contract_stage_ids()` covers the published FASTQ stage contracts.
`implemented_stages()` covers the closed execution subset implemented for governed FASTQ stages in
`bijux-dna-stages-fastq`.
`closed_execution_stage_ids()` exposes the broader closed execution subset owned by the FASTQ
domain.
`observer_specialized_stage_ids()` is the narrower fully observer-specialized subset documented in
`OBSERVERS.md`.
`observer_stage_ids()` remains a compatibility alias for that observer-specialized subset.

## Registry completeness
`tests/contracts/registry_completeness.rs` ensures every domain stage appears in the stage registry.
When adding a stage, update the registry and this document.

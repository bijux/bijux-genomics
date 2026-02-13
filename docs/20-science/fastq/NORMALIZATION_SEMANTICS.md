# Normalization Semantics

## Purpose
Define abundance normalization semantics for eDNA and pollen FASTQ analyses.

## Scope
`fastq.abundance_normalization` outputs and interpretation boundaries.

## Non-goals
- Claiming absolute abundance from compositional data.
- Prescribing one normalization transform for all markers.

## Contracts
- Normalized outputs are explicitly labeled as relative/compositional.
- The selected transform and parameters are emitted in artifacts.
- Downstream comparisons must use the same normalization contract.

## Examples
- Relative-abundance normalization for community composition summaries.
- Presence/absence thresholds for robust low-depth comparisons.

## Failure modes
- Cross-study normalization mismatch introduces artificial shifts.
- Rare taxa can be unstable under aggressive filtering.

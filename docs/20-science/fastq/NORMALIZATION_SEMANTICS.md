# Normalization Semantics

## Purpose
Define abundance normalization semantics for eDNA and pollen FASTQ analyses.

## Scope
`fastq.normalize_abundance` outputs and interpretation boundaries.

## Non-goals
- Claiming absolute abundance from compositional data.
- Prescribing one normalization transform for all markers.

## Contracts
- The abundance-normalization stage contract lives in
  [domain/fastq/stages/normalize_abundance.yaml](../../../domain/fastq/stages/normalize_abundance.yaml).
- Metric names and normalization observability fields live in
  [domain/fastq/metrics.yaml](../../../domain/fastq/metrics.yaml).
- The pinned default backend and method policy live in
  [domain/fastq/docs/DEFAULT_SETTINGS.md](../../../domain/fastq/docs/DEFAULT_SETTINGS.md).

## Examples
- Relative-abundance normalization for community composition summaries.
- Presence/absence thresholds for robust low-depth comparisons.

## Failure modes
- Cross-study normalization mismatch introduces artificial shifts.
- Rare taxa can be unstable under aggressive filtering.
- Treating compositional outputs as absolute abundance causes invalid ecological claims.

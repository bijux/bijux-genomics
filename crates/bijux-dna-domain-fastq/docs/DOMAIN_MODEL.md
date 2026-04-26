# bijux-dna-domain-fastq Domain Model

This crate owns FASTQ domain truth: IDs, stage contracts, bank contracts, parameters, metrics,
invariants, observer semantics, and benchmark metadata.

## Stage Truth

- Stage IDs and constants live in `src/stages/ids.rs`.
- Stage IO, semantics, stable ordering, criticality, and contract JSON live under `src/stages/`.
- Scientific intent is explicit: validate malformed inputs, profile reads, correct or transform
  reads, trim adapters and low-quality bases, merge pairs, deplete contaminants, screen taxonomy,
  and report QC.

## Parameters

- Parameter descriptors live under `src/params/descriptor/`.
- Effective defaults live under `src/params/defaults/`.
- Stage-specific parameter families live under `src/params/processing/`, `src/params/quality/`,
  and `src/params/edna.rs`.
- Canonicalization must preserve deterministic key ordering, float rendering, and path treatment.

## Metrics And Invariants

- Metric type contracts live under `src/metrics/types/`.
- Metric classes and specs live under `src/metrics/spec/`.
- Invariant specs and threshold evaluation live under `src/invariants/`.
- Retention is always a stage-boundary ratio with numerator, denominator, unit, conditions, and
  boundary context. Naked percentages are forbidden.

## Banks And References

- Adapter banks live under `src/banks/adapter/`.
- Contaminant banks live under `src/banks/contaminant/`.
- PolyX banks live under `src/banks/polyx/`.
- Selection policy lives under `src/banks/selection/` and must not be redefined downstream.
- Bank changes require provenance, stable ordering, and refreshed fixtures or snapshots.

## Observer And Benchmark Contracts

- Observer contracts map stage/tool outputs onto semantic surfaces without executing tools.
- Benchmark query context and corpus metadata are domain descriptors, not benchmark execution.

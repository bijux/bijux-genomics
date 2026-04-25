# FASTQ Gold Pipeline Spec

## What
Defines gold‑standard FASTQ pipeline expectations.

## Why
Provides a reference baseline for audits.

## Non-goals
- Performance benchmarking.

## Contracts
- Pipeline defaults ledger.
- Generic default FASTQ profile requires validation, read-length profiling, adapter detection, polyG trimming, ordinary trimming/filtering, neutral read profiling, overrepresented-sequence profiling, and QC aggregation.
- Minimal FASTQ profile requires only validation, adapter detection, ordinary trimming/filtering, and QC aggregation.
- Terminal-damage trimming is outside generic defaults and belongs to aDNA/reference-aDNA profiles or explicit user selection.
- Inline UMI extraction, when requested, is upstream of trimming and filtering.

## Examples
- Default pipeline profile used for regression checks.

## Failure modes
- Drift from gold defaults requires explicit approval.

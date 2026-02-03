# FASTQ Metric Layer v1

This document defines the FASTQ metric layer for bijux-dna. It is a scientific contract.

## Supported stages (v1)

- `fastq.trim` — trimming and preprocessing
- `fastq.validate` — validation and basic statistics
- `fastq.filter` — quality-based filtering (schema defined; benchmark pending)
- `fastq.merge` — read merging (schema defined; benchmark pending)

## Stage schemas

### fastq.trim (FastqTrimMetrics v1)
- reads_in
- reads_out
- bases_in
- bases_out
- mean_q_before
- mean_q_after

Invariants:
- reads_out ≤ reads_in
- bases_out ≤ bases_in
- mean_q_after ≥ mean_q_before (warn)

### fastq.validate (FastqValidateMetrics v1)
- reads_total
- reads_valid
- reads_invalid
- mean_q

Invariants:
- reads_valid + reads_invalid == reads_total
- mean_q ∈ [0, 45]

### fastq.filter (FastqFilterMetrics v1)
- reads_in
- reads_out
- reads_dropped
- mean_q_before
- mean_q_after

Invariants:
- reads_out + reads_dropped == reads_in
- mean_q_after ≥ mean_q_before (warn)

### fastq.merge (FastqMergeMetrics v1)
- reads_r1
- reads_r2
- reads_merged
- reads_unmerged
- merge_rate

Invariants:
- reads_merged + reads_unmerged ≤ min(reads_r1, reads_r2)
- merge_rate ∈ [0, 1]

## Shared execution metrics

Every benchmark record includes:
- runtime_s (seconds)
- memory_mb (MB)
- exit_code (process exit status)

And a required context block:
- tool
- tool version
- image digest
- runner
- platform
- input hash
- parameters

## Schema introspection

Use the CLI to inspect schema details and invariants:

```bash
bijux bench schema fastq.trim
bijux bench schema fastq.validate
bijux bench schema fastq.filter
bijux bench schema fastq.merge
```

## Measurement methodology

- read counts and base counts: computed via `seqkit stats` on input/output FASTQ files.
- mean quality: taken from `seqkit stats` output (AvgQual column) for the same file.
- validation read counts: derived from tool output when available (e.g., fastqvalidator); otherwise default to input reads.
- merge counts: computed from merged and unmerged FASTQ outputs; merge_rate uses reads_merged / min(reads_r1, reads_r2).

## Derived comparability metrics (computed, not stored)

- read_retention: reads_out / reads_in (trim/filter).
- base_retention: bases_out / bases_in (trim).
- merge_efficiency: reads_merged / min(reads_r1, reads_r2) (merge).

## Known limitations

- Validation tools differ in how they count invalid reads; `reads_valid` may collapse to 0/total for pass/fail tools.
- Mean quality is computed via tool-specific logic and may differ slightly across tools.
- `fastq.filter` and `fastq.merge` are schema-stable but may not yet have full benchmark harness coverage.

# FastQ Merge Metrics Spec v1

This document defines the **FastQ Merge** benchmark schema and the rules that govern it. It is a contract.

## Scope

Applies to the `fastq.merge` stage and any tool that emits `FastqMergeMetrics` v1.

## Metrics

### reads_r1
- **Definition:** Number of reads in the read 1 input FASTQ.
- **Type:** `u64`
- **Measurement:** `seqkit stats -a -T` on read 1, `num_seqs` column.

### reads_r2
- **Definition:** Number of reads in the read 2 input FASTQ.
- **Type:** `u64`
- **Measurement:** `seqkit stats -a -T` on read 2, `num_seqs` column.

### reads_merged
- **Definition:** Number of merged reads produced by the tool.
- **Type:** `u64`
- **Measurement:** `seqkit stats -a -T` on the merged FASTQ output, `num_seqs` column.

### reads_unmerged
- **Definition:** Number of unmerged reads (per end).
- **Type:** `u64`
- **Measurement:** `seqkit stats -a -T` on the unmerged read 1 output, `num_seqs` column.

### merge_rate
- **Definition:** Fraction of reads merged relative to the smaller input.
- **Type:** `f64`
- **Measurement:** `reads_merged / min(reads_r1, reads_r2)`.

## Invariants

- `reads_merged + reads_unmerged <= min(reads_r1, reads_r2)`
- `merge_rate ∈ [0, 1]`
- counts are non-negative

Invalid metrics **must** hard-fail validation.

## Tool coverage

- pear
- vsearch
- bbmerge
- flash2

## Measurement details

- Input FASTQ files are hashed with SHA-256; the combined digest is stored as `input_hash`.
- Metrics are measured with `seqkit` inside the configured container image.
- For tools that produce only merged or only unmerged outputs, metrics are derived from the available files.

## Known limitations

- Merge rate is not a quality metric and is not used for ranking by itself.
- Some tools emit additional reports not captured here.

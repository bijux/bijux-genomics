# FastQ Filter Metrics Spec v1

This document defines the **FastQ Filter** benchmark schema and the rules that govern it. It is a contract.

## Scope

Applies to the `fastq.filter` stage and any tool that emits `FastqFilterMetrics` v1.

## Metrics

### reads_in
- **Definition:** Number of reads in the input FASTQ.
- **Type:** `u64`
- **Measurement:** `seqkit stats -a -T` on the input file, `num_seqs` column.

### reads_out
- **Definition:** Number of reads in the filtered FASTQ output.
- **Type:** `u64`
- **Measurement:** `seqkit stats -a -T` on the output file, `num_seqs` column.

### reads_dropped
- **Definition:** Number of reads removed by filtering.
- **Type:** `u64`
- **Measurement:** `reads_in - reads_out`.

### mean_q_before
- **Definition:** Mean base quality score in the input FASTQ.
- **Type:** `f64`
- **Measurement:** `seqkit stats -a -T` on the input file, `avg_qual` column.

### mean_q_after
- **Definition:** Mean base quality score in the filtered FASTQ output.
- **Type:** `f64`
- **Measurement:** `seqkit stats -a -T` on the output file, `avg_qual` column.

## Invariants

- `reads_out + reads_dropped == reads_in`
- `mean_q_after >= mean_q_before` (warning only)
- counts are non-negative

Invalid metrics **must** hard-fail validation.

## Tool coverage

- prinseq
- fastp
- seqkit

## Measurement details

- Input FASTQ is hashed with SHA-256; the digest is stored as `input_hash`.
- Metrics are measured with `seqkit` inside the configured container image.
- For tools that emit uncompressed output (e.g., `prinseq`), the output path is normalized before measurement.

## Known limitations

- Mean quality is a single scalar and cannot capture distributional changes.
- Some tools may reorder reads; this does not affect the metrics.

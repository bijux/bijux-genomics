# FastQ Trim Metrics v1

Version: 1
Stage: fastq.trim

## Purpose
This specification defines the exact metrics recorded for FASTQ trimming benchmarks. It is a fixed contract for measurement, validation, and comparison across tools.

## Metrics

### reads_in (u64)
Number of reads in the input FASTQ. Measured by `seqkit stats -a -T` on the input file and reading the `num_seqs` column.

### reads_out (u64)
Number of reads in the trimmed FASTQ output. Measured by `seqkit stats -a -T` on the output file and reading the `num_seqs` column.

### bases_in (u64)
Total bases in the input FASTQ. Measured by `seqkit stats -a -T` on the input file and reading the `sum_len` column.

### bases_out (u64)
Total bases in the trimmed FASTQ output. Measured by `seqkit stats -a -T` on the output file and reading the `sum_len` column.

### mean_q_before (f64)
Mean base quality score in the input FASTQ. Measured by `seqkit stats -a -T` on the input file and reading the `avg_qual` column.

### mean_q_after (f64)
Mean base quality score in the trimmed FASTQ output. Measured by `seqkit stats -a -T` on the output file and reading the `avg_qual` column.

## Invariants

- reads_out <= reads_in (hard fail)
- bases_out <= bases_in (hard fail)
- mean_q_after >= mean_q_before (warning only)
- All counts are non-negative by type (u64)

## Measurement details

- The input FASTQ for trimming is hashed with SHA-256. The hex digest is stored as the input hash for cross-tool comparability.
- Metrics are measured with `seqkit` inside the configured container image. Only the input and output FASTQ files are mounted into the container.
- The same input metrics are used for every tool in a benchmark run.

## Exclusions (intentional)

- Adapter sequences removed or retained (not measured)
- Per-cycle quality or length distributions (not measured)
- Duplicate rates or read pairing statistics (not measured)
- Tool-specific logs or diagnostics (not stored in metrics)

## Known limitations

- Metrics depend on `seqkit` parsing and its interpretation of FASTQ quality encoding.
- Mean quality is a single scalar and cannot capture distributional changes.
- When a tool writes multiple outputs or intermediate files, only the primary trimmed FASTQ is measured.

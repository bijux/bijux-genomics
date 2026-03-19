# Bijux Run Report

- Run ID: `fastq-to-fastq__default__v1`
- Stages: 6
- Completeness: `incomplete`
- Pipeline Verdict: `Pass`

## Stage Summary
- `fastq.detect_adapters` via `fastqc` (99.99.99+fixture)
- `fastq.filter_reads` via `fastp` (99.99.99+fixture)
- `fastq.report_qc` via `multiqc` (99.99.99+fixture)
- `fastq.profile_reads` via `seqkit_stats` (99.99.99+fixture)
- `fastq.trim_reads` via `fastp` (99.99.99+fixture)
- `fastq.validate_reads` via `fastqvalidator` (99.99.99+fixture)
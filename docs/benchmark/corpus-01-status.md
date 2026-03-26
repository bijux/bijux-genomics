# `corpus-01` FASTQ benchmark publication status

- Stage count: `10`
- Completed stage dossiers: `8`
- Publication issues: `2`

## Stage status

- `fastq.validate_reads`: `complete` (`0` issues)
- `fastq.detect_adapters`: `complete` (`0` issues)
- `fastq.merge_pairs`: `complete` (`0` issues)
- `fastq.profile_reads`: `complete` (`0` issues)
- `fastq.profile_read_lengths`: `complete` (`0` issues)
- `fastq.profile_overrepresented_sequences`: `complete` (`0` issues)
- `fastq.trim_polyg_tails`: `complete` (`0` issues)
- `fastq.trim_reads`: `incomplete` (`1` issues)
  - `missing-corpus-dir`: missing benchmark/fastq.trim_reads/corpus-01
- `fastq.trim_terminal_damage`: `incomplete` (`1` issues)
  - `missing-corpus-dir`: missing benchmark/fastq.trim_terminal_damage/corpus-01
- `fastq.report_qc`: `complete` (`0` issues)

## Contract

A complete published corpus dossier requires `corpus-01-method.md`, `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`, and `lunarc.md`.

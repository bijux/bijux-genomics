# `corpus-01` FASTQ benchmark publication status

- Benchmarkable governed stages: `23`
- Corpus-applicable publication stages: `20`
- Completed stage dossiers: `10`
- Incomplete stage dossiers: `10`
- Excluded stages: `3`
- Publication issues: `10`

## Stage status

- `fastq.validate_reads`: `complete` (`0` issues, scope `full`)
- `fastq.detect_adapters`: `complete` (`0` issues, scope `full`)
- `fastq.profile_reads`: `complete` (`0` issues, scope `full`)
- `fastq.profile_read_lengths`: `complete` (`0` issues, scope `full`)
- `fastq.profile_overrepresented_sequences`: `complete` (`0` issues, scope `full`)
- `fastq.normalize_primers`: `incomplete` (`1` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.normalize_primers/corpus-01
- `fastq.trim_polyg_tails`: `complete` (`0` issues, scope `full`)
- `fastq.trim_reads`: `complete` (`0` issues, scope `full`)
- `fastq.filter_reads`: `incomplete` (`1` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.filter_reads/corpus-01
- `fastq.filter_low_complexity`: `incomplete` (`1` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.filter_low_complexity/corpus-01
- `fastq.deplete_rrna`: `incomplete` (`1` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.deplete_rrna/corpus-01
- `fastq.merge_pairs`: `complete` (`0` issues, scope `paired`)
- `fastq.remove_duplicates`: `incomplete` (`1` issues, scope `paired`)
  - `missing-corpus-dir`: missing benchmark/fastq.remove_duplicates/corpus-01
- `fastq.deplete_host`: `incomplete` (`1` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.deplete_host/corpus-01
- `fastq.deplete_reference_contaminants`: `incomplete` (`1` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.deplete_reference_contaminants/corpus-01
- `fastq.correct_errors`: `incomplete` (`1` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.correct_errors/corpus-01
- `fastq.extract_umis`: `incomplete` (`1` issues, scope `paired`)
  - `missing-corpus-dir`: missing benchmark/fastq.extract_umis/corpus-01
- `fastq.screen_taxonomy`: `incomplete` (`1` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.screen_taxonomy/corpus-01
- `fastq.trim_terminal_damage`: `complete` (`0` issues, scope `full`)
- `fastq.report_qc`: `complete` (`0` issues, scope `full`)

## Excluded Stages

- `fastq.index_reference`: Reference-index benchmarking measures bundle build throughput rather than sample-cohort execution on corpus-01 FASTQ inputs.
- `fastq.cluster_otus`: OTU clustering is amplicon-specific and does not fit the human whole-genome FASTQ cohort contract used by corpus-01.
- `fastq.normalize_abundance`: Abundance normalization benchmarks require derived abundance tables rather than the raw FASTQ corpus-01 publication surface.

## Contract

A complete published corpus dossier requires `corpus-01-method.md`, `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`, and `lunarc.md`.
Published summaries must also match the governed scenario id, exact benchmark tool roster, expected corpus scope (`full` or `paired`), zero sample failures, and complete sample-by-tool coverage.

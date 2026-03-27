# `corpus-01` FASTQ benchmark publication status

- Benchmarkable governed stages: `23`
- Corpus-applicable publication stages: `20`
- Completed stage dossiers: `15`
- Incomplete stage dossiers: `5`
- Excluded stages: `3`
- Publication issues: `10`

## Stage status

- `fastq.validate_reads`: `complete` (`0` issues, scope `full`)
- `fastq.detect_adapters`: `complete` (`0` issues, scope `full`)
- `fastq.profile_reads`: `complete` (`0` issues, scope `full`)
- `fastq.profile_read_lengths`: `complete` (`0` issues, scope `full`)
- `fastq.profile_overrepresented_sequences`: `complete` (`0` issues, scope `full`)
- `fastq.normalize_primers`: `incomplete` (`1` issues, scope `full`)
  - `publication-toolset-subset`: fastq.normalize_primers publication contract covers ['cutadapt'] but governed stage toolset also admits ['seqkit']
- `fastq.trim_polyg_tails`: `complete` (`0` issues, scope `full`)
- `fastq.trim_reads`: `incomplete` (`1` issues, scope `full`)
  - `publication-toolset-subset`: fastq.trim_reads publication contract covers ['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] but governed stage toolset also admits ['alientrimmer', 'fastx_clipper', 'leehom', 'skewer']
- `fastq.filter_reads`: `complete` (`0` issues, scope `full`)
- `fastq.filter_low_complexity`: `complete` (`0` issues, scope `full`)
- `fastq.deplete_rrna`: `complete` (`0` issues, scope `full`)
- `fastq.merge_pairs`: `complete` (`0` issues, scope `paired`)
- `fastq.remove_duplicates`: `complete` (`0` issues, scope `paired`)
- `fastq.deplete_host`: `incomplete` (`2` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.deplete_host/corpus-01
  - `synthetic-host-reference-lineage`: artifacts/reference_store/Homo sapiens/GRCh38/refs/raw/reference.fa.gz is an ASCII synthetic placeholder payload rather than a real compressed reference, so corpus-01 deplete_host cannot publish against that lineage.
- `fastq.deplete_reference_contaminants`: `complete` (`0` issues, scope `full`)
- `fastq.correct_errors`: `incomplete` (`4` issues, scope `paired`)
  - `publication-toolset-subset`: fastq.correct_errors publication contract covers ['lighter', 'musket', 'rcorrector'] but governed stage toolset also admits ['bayeshammer']
  - `missing-corpus-dir`: missing benchmark/fastq.correct_errors/corpus-01
  - `bayeshammer-retention-contract-drift`: the governed BayesHammer path on Lunarc drops reads on paired corpus-01 inputs, which violates the fastq.correct_errors retention contract (`may_change_read_count = false`), so BayesHammer cannot be counted as a corpus-complete governed benchmark backend yet.
  - `musket-kmer-budget-unmapped`: the governed Musket adapter still emits `musket -k <kmer_size>` even though Musket 1.1 requires `-k <kmer_size> <estimated_total_kmers>`, so the current corpus-01 benchmark path cannot yet produce valid Musket corrected outputs.
- `fastq.extract_umis`: `complete` (`0` issues, scope `paired`)
- `fastq.screen_taxonomy`: `incomplete` (`2` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.screen_taxonomy/corpus-01
  - `minimal-taxonomy-database-lineage`: the available Lunarc cache lineage bijux-reference/taxonomy/minimal_screen_v1 is a PhiX174/UniVec smoke-test database, so corpus-01 screen_taxonomy cannot publish against it as a full taxonomy benchmark.
- `fastq.trim_terminal_damage`: `complete` (`0` issues, scope `full`)
- `fastq.report_qc`: `complete` (`0` issues, scope `full`)

## Excluded Stages

- `fastq.index_reference`: Reference-index benchmarking measures bundle build throughput rather than sample-cohort execution on corpus-01 FASTQ inputs.
- `fastq.cluster_otus`: OTU clustering is amplicon-specific and does not fit the human whole-genome FASTQ cohort contract used by corpus-01.
- `fastq.normalize_abundance`: Abundance normalization benchmarks require derived abundance tables rather than the raw FASTQ corpus-01 publication surface.

## Contract

A complete published corpus dossier requires `corpus-01-method.md`, `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`, and `lunarc.md`.
Published summaries must also match the governed scenario id, exact benchmark tool roster, expected corpus scope (`full` or `paired`), zero sample failures, and complete sample-by-tool coverage.

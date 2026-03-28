# `corpus-01` FASTQ benchmark publication status

- Benchmarkable governed stages: `23`
- Corpus-applicable publication stages: `20`
- Completed stage dossiers: `16`
- Incomplete stage dossiers: `4`
- Excluded stages: `3`
- Publication issues: `29`

## Stage status

- `fastq.validate_reads`: `complete` (`0` issues, scope `full`)
- `fastq.detect_adapters`: `complete` (`0` issues, scope `full`)
- `fastq.profile_reads`: `complete` (`0` issues, scope `full`)
- `fastq.profile_read_lengths`: `complete` (`0` issues, scope `full`)
- `fastq.profile_overrepresented_sequences`: `complete` (`0` issues, scope `full`)
- `fastq.normalize_primers`: `complete` (`0` issues, scope `full`)
- `fastq.trim_polyg_tails`: `complete` (`0` issues, scope `full`)
- `fastq.trim_reads`: `incomplete` (`25` issues, scope `full`)
  - `summary-tool-roster-drift`: benchmark/fastq.trim_reads/corpus-01/summary.json tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `summary-tool-summary-drift`: benchmark/fastq.trim_reads/corpus-01/summary.json tool_summary tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-roster-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0001 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0002 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0003 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0004 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0005 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0006 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0007 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0008 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0009 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0010 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0011 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0012 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0013 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0014 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0015 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0016 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0017 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0018 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0019 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-tool-coverage-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv sample sample_0020 tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
  - `sample-results-row-count-drift`: benchmark/fastq.trim_reads/corpus-01/sample_results.csv row_count=180 expected 260
  - `tool-runtime-summary-drift`: benchmark/fastq.trim_reads/corpus-01/tool_runtime_summary.csv tools=['adapterremoval', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'prinseq', 'seqkit', 'trim_galore', 'trimmomatic'] expected ['adapterremoval', 'alientrimmer', 'atropos', 'bbduk', 'cutadapt', 'fastp', 'fastx_clipper', 'leehom', 'prinseq', 'seqkit', 'skewer', 'trim_galore', 'trimmomatic']
- `fastq.filter_reads`: `complete` (`0` issues, scope `full`)
- `fastq.filter_low_complexity`: `complete` (`0` issues, scope `full`)
- `fastq.deplete_rrna`: `complete` (`0` issues, scope `full`)
- `fastq.merge_pairs`: `complete` (`0` issues, scope `paired`)
- `fastq.remove_duplicates`: `complete` (`0` issues, scope `paired`)
- `fastq.deplete_host`: `incomplete` (`1` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.deplete_host/corpus-01
- `fastq.deplete_reference_contaminants`: `complete` (`0` issues, scope `full`)
- `fastq.correct_errors`: `incomplete` (`1` issues, scope `paired`)
  - `missing-corpus-dir`: missing benchmark/fastq.correct_errors/corpus-01
- `fastq.extract_umis`: `complete` (`0` issues, scope `paired`)
- `fastq.screen_taxonomy`: `incomplete` (`2` issues, scope `full`)
  - `missing-corpus-dir`: missing benchmark/fastq.screen_taxonomy/corpus-01
  - `minimal-taxonomy-database-lineage`: corpus-01 screen_taxonomy still lacks a materialized governed taxonomy bundle under /home/bijan/lu2024-12-24/.cache/extra-data/benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db with backend directories and lineage.json; the legacy minimal smoke-test database is not sufficient for honest publication.
- `fastq.trim_terminal_damage`: `complete` (`0` issues, scope `full`)
- `fastq.report_qc`: `complete` (`0` issues, scope `full`)

## Excluded Stages

- `fastq.index_reference`: Reference-index benchmarking measures bundle build throughput rather than sample-cohort execution on corpus-01 FASTQ inputs.
- `fastq.cluster_otus`: OTU clustering is amplicon-specific and does not fit the human whole-genome FASTQ cohort contract used by corpus-01.
- `fastq.normalize_abundance`: Abundance normalization benchmarks require derived abundance tables rather than the raw FASTQ corpus-01 publication surface.

## Contract

A complete published corpus dossier requires `corpus-01-method.md`, `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`, and `lunarc.md`.
Published summaries must also match the governed scenario id, exact benchmark tool roster, expected corpus scope (`full` or `paired`), zero sample failures, and complete sample-by-tool coverage.

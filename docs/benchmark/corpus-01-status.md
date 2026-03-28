# `corpus-01` FASTQ benchmark publication status

- Benchmarkable governed stages: `23`
- Corpus-applicable publication stages: `20`
- Completed stage dossiers: `18`
- Incomplete stage dossiers: `2`
- Excluded stages: `3`
- Publication issues: `26`
- Audit warnings: `0`

## Stage status

- `fastq.validate_reads`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.validate_reads/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.validate_reads/lunarc` (selected newest=`True`)
- `fastq.detect_adapters`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.detect_adapters/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.detect_adapters/lunarc` (selected newest=`True`)
- `fastq.profile_reads`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_reads/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_reads/lunarc` (selected newest=`True`)
- `fastq.profile_read_lengths`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_read_lengths/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_read_lengths/lunarc` (selected newest=`True`)
- `fastq.profile_overrepresented_sequences`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_overrepresented_sequences/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.profile_overrepresented_sequences/lunarc` (selected newest=`True`)
- `fastq.normalize_primers`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.normalize_primers/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.normalize_primers/lunarc` (selected newest=`True`)
- `fastq.trim_polyg_tails`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_polyg_tails/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_polyg_tails/lunarc` (selected newest=`True`)
- `fastq.trim_reads`: `incomplete` (`25` publication issues, results `incomplete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_reads/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/results/corpus_01/fastq.trim_reads/lunarc` (selected newest=`False`)
  - mirrored result issues: `4`
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
- `fastq.filter_reads`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.filter_reads/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.filter_reads/lunarc` (selected newest=`True`)
- `fastq.filter_low_complexity`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.filter_low_complexity/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.filter_low_complexity/lunarc` (selected newest=`True`)
- `fastq.deplete_rrna`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.deplete_rrna/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.deplete_rrna/lunarc` (selected newest=`True`)
- `fastq.merge_pairs`: `complete` (`0` publication issues, results `complete`, scope `paired`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.merge_pairs/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.merge_pairs/lunarc` (selected newest=`True`)
- `fastq.remove_duplicates`: `complete` (`0` publication issues, results `complete`, scope `paired`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.remove_duplicates/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.remove_duplicates/lunarc` (selected newest=`True`)
- `fastq.deplete_host`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.deplete_host/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.deplete_host/lunarc` (selected newest=`True`)
- `fastq.deplete_reference_contaminants`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.deplete_reference_contaminants/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.deplete_reference_contaminants/lunarc` (selected newest=`True`)
- `fastq.correct_errors`: `incomplete` (`1` publication issues, results `incomplete`, scope `paired`)
  - mirrored result issues: `1`
  - `missing-corpus-dir`: missing benchmark/fastq.correct_errors/corpus-01
- `fastq.extract_umis`: `complete` (`0` publication issues, results `complete`, scope `paired`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.extract_umis/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.extract_umis/lunarc` (selected newest=`True`)
- `fastq.screen_taxonomy`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/results/corpus_01/fastq.screen_taxonomy/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/home/bijan/lu2024-12-24/.cache/results/corpus_01/fastq.screen_taxonomy/lunarc` (selected newest=`True`)
- `fastq.trim_terminal_damage`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_terminal_damage/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.trim_terminal_damage/lunarc` (selected newest=`True`)
- `fastq.report_qc`: `complete` (`0` publication issues, results `complete`, scope `full`)
  - selected mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.report_qc/lunarc`
  - newest mirrored run root: `/Users/bijan/bijux/bijux-dna-results/corpus_01/fastq.report_qc/lunarc` (selected newest=`True`)

## Excluded Stages

- `fastq.index_reference`: Reference-index benchmarking measures bundle build throughput rather than sample-cohort execution on corpus-01 FASTQ inputs.
- `fastq.cluster_otus`: OTU clustering is amplicon-specific and does not fit the human whole-genome FASTQ cohort contract used by corpus-01.
- `fastq.normalize_abundance`: Abundance normalization benchmarks require derived abundance tables rather than the raw FASTQ corpus-01 publication surface.

## Contract

A complete published corpus dossier requires `corpus-01-method.md`, `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`, and `benchmark.md`.
Published summaries must also match the governed scenario id, exact benchmark tool roster, expected corpus scope (`full` or `paired`), zero sample failures, and complete sample-by-tool coverage.

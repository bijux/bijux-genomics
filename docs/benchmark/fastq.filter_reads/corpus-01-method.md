# `fastq.filter_reads` corpus-01 method

## Scope
- Stage: `fastq.filter_reads`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `filter_fairness`

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.filter_reads --kind benchmark`.
- The current governed fairness cohort is:
  - `bbduk`
  - `fastp`
  - `prinseq`
  - `seqkit`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Pin one governed filter contract across the whole corpus:
  - identical input hashes for every backend
  - identical filter-threshold contract hash
  - identical stage defaults inherited from the benchmark scenario
- Preserve backend-native filter reports so retention and rejection behavior remain auditable.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and retention summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or lowest-retention samples.
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.filter_reads
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.filter_reads
```

The runner resolves the governed corpus root through `configs/bench/benchmark.toml`. Change that config or pass `--config` only when you intentionally target a different governed workspace.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier published from `--dry-run` or `--sample-limit`.
- Reject any dossier that omits a tool row for any sample.
- Reject any dossier whose retention metrics drift from the governed filter contract.

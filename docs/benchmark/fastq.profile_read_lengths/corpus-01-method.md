# `fastq.profile_read_lengths` corpus-01 method

## Scope
- Stage: `fastq.profile_read_lengths`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `read_length_fairness`

## Governed tool cohort
- The benchmark runner resolves the tool roster from `bijux-dna registry list-tools --stage fastq.profile_read_lengths --kind benchmark`.
- The current governed fairness cohort is:
  - `seqkit_stats`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Pin the governed read-length profile contract across the whole corpus:
  - `report_only = true`
  - `mutates_fastq = false`
  - `may_change_read_count = false`
  - `raw_backend_report_format = seqkit_stats_length_histogram`
  - `histogram_bins = 100`
  - histogram artifacts:
    - `profile_read_lengths_report.json`
    - `length_distribution.tsv`
    - `length_distribution.json`

## Why this stage is benchmarked differently
- `fastq.profile_read_lengths` does not modify FASTQ content.
- The benchmark therefore measures runtime and deterministic length-distribution stability, not retention deltas.
- This dossier is a corpus-wide baseline for future regressions and future backend additions to the governed cohort.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and read-length summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or widest-length-support samples.
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.profile_read_lengths
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.profile_read_lengths
```

The default corpus root is loaded from `configs/bench/benchmark.toml`. Update that config or pass `--config` when rerendering against a different governed workspace.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any published report produced from `--dry-run`.
- Reject any published report that omits a tool row for any sample.
- Reject any published report that lacks the governed histogram artifacts.
- Reject any published report that carries invalid read-count or length-distribution metrics.

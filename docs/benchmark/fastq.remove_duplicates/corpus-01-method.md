# `fastq.remove_duplicates` corpus-01 method

## Scope
- Stage: `fastq.remove_duplicates`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `dedup_fairness`
- Sample scope: paired-end subset only

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.remove_duplicates --kind benchmark`.
- The current governed fairness cohort is:
  - `clumpify`
  - `fastuniq`

## Execution contract
- Use only the paired-end subset of `corpus-01/normalized/`.
- Require the balanced paired corpus contract:
  - `5` ancient paired-end
  - `5` modern paired-end
- Hold one governed duplicate-removal contract across the paired cohort:
  - identical paired input hashes
  - identical dedup policy hash
  - identical keep-order policy where the governed surface requires it
- Preserve pair synchronization and backend-native duplicate metrics in every successful sample report.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and retention summary.
- `cohort_runtime_summary.csv`: paired-era breakdowns and size-band rollups.
- `sample_runtime_outliers.csv`: slowest or lowest-retention samples.
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.remove_duplicates
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.remove_duplicates
```

The runner resolves the governed paired corpus root through `configs/bench/benchmark.toml`. Change that config or pass `--config` only when you intentionally target a different governed workspace.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that contains single-end samples.
- Reject any dossier that omits a tool row for any paired sample.

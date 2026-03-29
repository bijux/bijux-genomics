# `fastq.screen_taxonomy` corpus-01 method

## Scope
- Stage: `fastq.screen_taxonomy`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `screen_fairness`

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.screen_taxonomy --kind benchmark`.
- The current governed fairness cohort is:
  - `centrifuge`
  - `kaiju`
  - `kraken2`
  - `krakenuniq`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Hold the governed taxonomy database lineage constant across the full corpus:
  - identical input hashes
  - identical contamination database digest
  - identical database namespace and scope
  - identical governed taxonomy normalization contract
- Preserve classifier-native reports and normalized assignment summaries for every successful sample row.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and classification summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or highest-contamination samples.
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.screen_taxonomy
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.screen_taxonomy
```

The runner resolves the governed taxonomy bundle from `configs/bench/benchmark.toml` unless you intentionally override `DATABASE_ROOT` or `--database-root`. Use `bijux-dna bench write-screen-taxonomy-database-lineage` to validate a chosen bundle and write `lineage.json` before benchmarking it.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that omits the governed database lineage from the run manifest.
- Reject any dossier that omits a tool row for any sample.

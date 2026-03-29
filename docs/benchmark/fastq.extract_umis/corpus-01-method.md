# `fastq.extract_umis` corpus-01 method

## Scope
- Stage: `fastq.extract_umis`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `umi_extraction_fairness`
- Sample scope: paired-end subset only

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.extract_umis --kind benchmark`.
- The current governed fairness cohort is:
  - `umi_tools`

## Execution contract
- Use only the paired-end subset of `corpus-01/normalized/`.
- Require the balanced paired corpus contract:
  - `5` ancient paired-end
  - `5` modern paired-end
- Hold one governed UMI extraction policy across the full paired cohort:
  - identical paired input hashes
  - identical UMI pattern and parsing policy
  - identical missing-header bypass policy when `corpus-01` inputs do not carry native UMI headers
  - identical mate-synchronization contract for retained reads
- Preserve governed UMI reports and extracted read paths in every successful sample row.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and UMI extraction summary.
- `cohort_runtime_summary.csv`: paired-era breakdowns and size-band rollups.
- `sample_runtime_outliers.csv`: slowest or most disruptive samples.
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.extract_umis
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.extract_umis
```

The runner resolves the governed paired corpus root through `configs/bench/benchmark.toml`. Change that config or pass `--config` only when you intentionally target a different governed workspace.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that contains single-end samples.
- Reject any dossier that does not record whether missing-header bypass was enabled.
- Reject any dossier that omits a tool row for any paired sample.

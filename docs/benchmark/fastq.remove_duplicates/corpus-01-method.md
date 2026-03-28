# `fastq.remove_duplicates` corpus-01 method

## Scope
- Stage: `fastq.remove_duplicates`
- Corpus: `corpus-01`
- Platform target: `lunarc-apptainer`
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
- `lunarc.md`: narrative benchmark dossier for the Lunarc run.

## Workflow
```bash
make _benchmark-remove-duplicates-corpus-01 PLATFORM=lunarc-apptainer
make _benchmark-remove-duplicates-corpus-01-report
```

The runner resolves the governed paired corpus root through `configs/bench/workspace.toml`. Override `CORPUS_ROOT` or `--corpus-root` only when you intentionally audit a non-governed mirror.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that contains single-end samples.
- Reject any dossier that omits a tool row for any paired sample.

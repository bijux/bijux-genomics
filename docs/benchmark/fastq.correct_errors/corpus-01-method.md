# `fastq.correct_errors` corpus-01 method

## Scope
- Stage: `fastq.correct_errors`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `correction_fairness`
- Sample scope: paired-end subset only

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.correct_errors --kind benchmark`.
- The current governed fairness cohort is:
  - `bayeshammer`
  - `lighter`
  - `musket`
  - `rcorrector`

## Execution contract
- Use only the paired-end subset of `corpus-01/normalized/`.
- Require the balanced paired corpus contract:
  - `5` ancient paired-end
  - `5` modern paired-end
- Hold one governed correction contract across the paired cohort:
  - identical input hashes
  - identical correction-policy hash
  - identical derived genome-size and k-mer inputs wherever the governed backend contract requires them
- Preserve backend-native correction reports so read-retention and correction-rate claims remain auditable.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and correction summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or most disruptive samples.
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
make _benchmark-correct-errors-corpus-01 PLATFORM=apptainer-amd64
make _benchmark-correct-errors-corpus-01-report
```

The runner resolves the governed paired corpus root through `configs/bench/benchmark.toml`. Override `CORPUS_ROOT` or `--corpus-root` only when you intentionally audit a non-governed mirror.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that contains single-end samples.
- Reject any dossier that omits a tool row for any paired sample.
- Reject any dossier whose correction-policy hash differs across tools or samples.

# `fastq.filter_low_complexity` corpus-01 method

## Scope
- Stage: `fastq.filter_low_complexity`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `low_complexity_fairness`

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.filter_low_complexity --kind benchmark`.
- The current governed fairness cohort is:
  - `bbduk`
  - `prinseq`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Hold one governed low-complexity policy across the full corpus:
  - identical input hashes for every backend
  - identical complexity-threshold contract hash
  - identical inherited benchmark defaults across the full roster
- Preserve backend-native complexity reports so entropy and sequence-mask behavior remain inspectable.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and retention summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or most aggressive samples.
- `lunarc.md`: narrative benchmark dossier for the Lunarc run.

## Workflow
```bash
make _benchmark-filter-low-complexity-corpus-01 PLATFORM=apptainer-amd64
make _benchmark-filter-low-complexity-corpus-01-report
```

The runner and report renderer resolve the governed corpus root through `configs/bench/workspace.toml`. Override `CORPUS_ROOT` or `--corpus-root` only when you intentionally audit a non-governed mirror.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that omits a tool row for any sample.
- Reject any dossier whose complexity-policy hash differs across tools or samples.

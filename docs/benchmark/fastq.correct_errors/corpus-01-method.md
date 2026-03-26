# `fastq.correct_errors` corpus-01 method

## Scope
- Stage: `fastq.correct_errors`
- Corpus: `corpus-01`
- Platform target: `lunarc-apptainer`
- Benchmark scenario: `correction_fairness`

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.correct_errors --kind benchmark`.
- The current governed fairness cohort is:
  - `bayeshammer`
  - `lighter`
  - `musket`
  - `rcorrector`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Hold one governed correction contract across the whole corpus:
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
- `lunarc.md`: narrative benchmark dossier for the Lunarc run.

## Publication gate
- This stage does not yet have a committed `corpus-01` runner and report renderer under `makes/bin/`.
- A publishable dossier begins once those entrypoints materialize `docs/benchmark/fastq.correct_errors/corpus-01/` under the audit contract described above.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that omits a tool row for any sample.
- Reject any dossier whose correction-policy hash differs across tools or samples.

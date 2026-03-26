# `fastq.deplete_rrna` corpus-01 method

## Scope
- Stage: `fastq.deplete_rrna`
- Corpus: `corpus-01`
- Platform target: `lunarc-apptainer`
- Benchmark scenario: `rrna_depletion_fairness`

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.deplete_rrna --kind benchmark`.
- The current governed fairness cohort is:
  - `sortmerna`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Hold the fixed governed rRNA bundle lineage across the whole corpus:
  - identical input hashes
  - identical rRNA reference bundle digest
  - identical depletion contract hash in the run manifest
- Preserve both retained reads and removed-read evidence where the governed stage contract emits them.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and depletion summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or most aggressive samples.
- `lunarc.md`: narrative benchmark dossier for the Lunarc run.

## Publication gate
- This stage does not yet have a committed `corpus-01` runner and report renderer under `makes/bin/`.
- A publishable dossier begins once those entrypoints materialize `docs/benchmark/fastq.deplete_rrna/corpus-01/` under the audit contract described above.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that omits the governed reference lineage from the run manifest.
- Reject any dossier that omits a tool row for any sample.

# `fastq.deplete_host` corpus-01 method

## Scope
- Stage: `fastq.deplete_host`
- Corpus: `corpus-01`
- Platform target: `lunarc-apptainer`
- Benchmark scenario: `host_depletion_fairness`

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.deplete_host --kind benchmark`.
- The current governed fairness cohort is:
  - `bowtie2`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Hold the governed host-reference lineage constant across the full corpus:
  - identical input hashes
  - identical host reference bundle digest
  - identical reference-index backend lineage
  - identical host-depletion policy hash
- Preserve both retained and removed-host read evidence wherever the governed stage contract emits them.

## Why this dossier matters on a human corpus
- `corpus-01` is intentionally human, so host depletion acts as a high-pressure control rather than a low-signal contamination screen.
- The published dossier must therefore explain not just runtime, but how aggressively the governed host-removal policy prunes the cohort.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and depletion summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or most aggressive samples.
- `lunarc.md`: narrative benchmark dossier for the Lunarc run.

## Publication gate
- This stage does not yet have a committed `corpus-01` runner and report renderer under `makes/bin/`.
- A publishable dossier begins once those entrypoints materialize `docs/benchmark/fastq.deplete_host/corpus-01/` under the audit contract described above.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that omits the governed host reference lineage or index provenance.
- Reject any dossier that omits a tool row for any sample.

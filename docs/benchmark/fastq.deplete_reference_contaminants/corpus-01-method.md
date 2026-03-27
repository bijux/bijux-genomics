# `fastq.deplete_reference_contaminants` corpus-01 method

## Scope
- Stage: `fastq.deplete_reference_contaminants`
- Corpus: `corpus-01`
- Platform target: `lunarc-apptainer`
- Benchmark scenario: `contaminant_depletion_fairness`

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.deplete_reference_contaminants --kind benchmark`.
- The current governed fairness cohort is:
  - `bowtie2`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Hold the contaminant-reference lineage constant across the full corpus:
  - identical input hashes
  - identical contaminant bundle digest
  - identical reference-index provenance
  - identical contaminant-depletion policy hash
- Preserve retained and removed-read evidence in every successful sample report.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and depletion summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or most aggressive samples.
- `lunarc.md`: narrative benchmark dossier for the Lunarc run.

## Publication gate
- Governed publication uses:
  - `makes/bin/run_fastq_deplete_reference_contaminants_corpus_01.py`
  - `makes/bin/render_fastq_deplete_reference_contaminants_corpus_01_report.py`
  - `makes/bin/render_fastq_deplete_reference_contaminants_corpus_01_briefing.py`
- A publishable dossier begins once an executed Lunarc run is rendered into `docs/benchmark/fastq.deplete_reference_contaminants/corpus-01/` under the audit contract described above.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that omits the governed contaminant bundle lineage or index provenance.
- Reject any dossier that omits a tool row for any sample.

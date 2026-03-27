# `fastq.extract_umis` corpus-01 method

## Scope
- Stage: `fastq.extract_umis`
- Corpus: `corpus-01`
- Platform target: `lunarc-apptainer`
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
  - identical mate-synchronization contract for retained reads
- Preserve governed UMI reports and extracted read paths in every successful sample row.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and UMI extraction summary.
- `cohort_runtime_summary.csv`: paired-era breakdowns and size-band rollups.
- `sample_runtime_outliers.csv`: slowest or most disruptive samples.
- `lunarc.md`: narrative benchmark dossier for the Lunarc run.

## Publication gate
- Governed publication uses:
  - `makes/bin/run_fastq_extract_umis_corpus_01.py`
  - `makes/bin/render_fastq_extract_umis_corpus_01_report.py`
  - `makes/bin/render_fastq_extract_umis_corpus_01_briefing.py`
- A publishable dossier begins once an executed Lunarc run is rendered into `docs/benchmark/fastq.extract_umis/corpus-01/` under the audit contract described above.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that contains single-end samples.
- Reject any dossier that omits a tool row for any paired sample.

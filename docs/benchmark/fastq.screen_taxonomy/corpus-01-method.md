# `fastq.screen_taxonomy` corpus-01 method

## Scope
- Stage: `fastq.screen_taxonomy`
- Corpus: `corpus-01`
- Platform target: `lunarc-apptainer`
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
- `lunarc.md`: narrative benchmark dossier for the Lunarc run.

## Publication gate
- Governed publication uses:
  - `makes/bin/bootstrap_fastq_screen_taxonomy_database.py`
  - `makes/bin/run_fastq_screen_taxonomy_corpus_01.py`
  - `makes/bin/render_fastq_screen_taxonomy_corpus_01_report.py`
  - `makes/bin/render_fastq_screen_taxonomy_corpus_01_briefing.py`
- If `--database-root` is omitted, the runner expects the governed default under `/home/bijan/lu2024-12-24/.cache/extra-data/benchmark/fastq.screen_taxonomy/<database_namespace>/<database_scope>/<database_artifact_id>/` on Lunarc.
- The taxonomy database path remains replaceable through `--database-root` or `BIJUX_TAXONOMY_DB`.
- The governed taxonomy bundle is expected to carry `lineage.json` at the database root so the run manifest can record both the database digest and the lineage digest.
- `bootstrap_fastq_screen_taxonomy_database.py` is the governed way to validate the built bundle and write `lineage.json` once the backend directories and `source/panel_manifest.json` are present under the chosen bundle root.
- A publishable dossier begins once an executed Lunarc run is rendered into `docs/benchmark/fastq.screen_taxonomy/corpus-01/` under the audit contract described above.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any dossier that omits the governed database lineage from the run manifest.
- Reject any dossier that omits a tool row for any sample.

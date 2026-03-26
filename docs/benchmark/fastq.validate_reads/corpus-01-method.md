# `fastq.validate_reads` corpus-01 method

## Scope
- Stage: `fastq.validate_reads`
- Corpus: `corpus-01`
- Platform target: `lunarc-apptainer`
- Benchmark scenario: `validation_fairness`

## Governed tool cohort
- The benchmark runner resolves the tool roster from `bijux-dna registry list-tools --stage fastq.validate_reads --kind benchmark`.
- The current governed fairness cohort is:
  - `fastq_scan`
  - `fastqc`
  - `fastqvalidator`
  - `fqtools`
  - `seqtk`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Run every tool against the same normalized reads without stage mutation.
- Preserve each tool's native validation behavior while normalizing the stage report contract.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest samples across the cohort.
- `lunarc.md`: narrative benchmark dossier for the Lunarc run.

## Workflow
```bash
make _benchmark-validate-reads-corpus-01 PLATFORM=lunarc-apptainer CORPUS_ROOT=/home/bijan/bijux/corpus_01
make _benchmark-validate-reads-corpus-01-report CORPUS_ROOT=/home/bijan/bijux/corpus_01
```

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any published report that omits a tool row for any sample.
- Preserve backend-native validation provenance through `raw_backend_report_format` when available.

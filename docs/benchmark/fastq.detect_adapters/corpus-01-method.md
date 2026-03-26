# `fastq.detect_adapters` corpus-01 method

## Scope
- Stage: `fastq.detect_adapters`
- Corpus: `corpus-01`
- Platform target: `lunarc-apptainer`
- Benchmark scenario: `detect_adapters_fairness`

## Governed tool cohort
- The benchmark runner resolves the tool roster from `bijux-dna registry list-tools --stage fastq.detect_adapters --kind benchmark`.
- The current governed fairness cohort is:
  - `fastqc`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Pin the governed observer policy across the whole corpus:
  - `inspection_mode = evidence_only`
  - `report_only = true`
  - `evidence_scope = full_input`
  - `evidence_format = fastqc_summary`
- Keep the benchmark evidence-only:
  - `reads_out == reads_in`
  - `bases_out == bases_in`

## Why this stage is benchmarked differently
- `fastq.detect_adapters` does not trim or filter reads.
- The benchmark therefore measures runtime, candidate-adapter signal, and evidence-contract stability rather than read-retention deltas.
- This dossier is a stability baseline for observer behavior across corpus composition until additional governed adapter-inspection backends join the benchmark cohort.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and signal summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or highest-signal samples.
- `lunarc.md`: narrative benchmark dossier for the Lunarc run.

## Workflow
```bash
make _benchmark-detect-adapters-corpus-01 PLATFORM=lunarc-apptainer CORPUS_ROOT=/home/bijan/bijux/corpus_01
make _benchmark-detect-adapters-corpus-01-report CORPUS_ROOT=/home/bijan/bijux/corpus_01
```

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any published report produced from `--dry-run`.
- Reject any published report whose rows drift from the governed observer contract.
- Reject any published report that omits a tool row for any sample.
- Reject any published report that mutates `reads_out` or `bases_out` relative to the input metrics.

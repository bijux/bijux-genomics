# `fastq.detect_adapters` corpus-01 method

## Scope
- Stage: `fastq.detect_adapters`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
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
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.detect_adapters
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.detect_adapters
```

The default corpus root is loaded from `configs/bench/benchmark.toml`. Update that config or pass `--config` when rerendering against a different governed workspace.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any published report produced from `--dry-run`.
- Reject any published report whose rows drift from the governed observer contract.
- Reject any published report that omits a tool row for any sample.
- Reject any published report that mutates `reads_out` or `bases_out` relative to the input metrics.

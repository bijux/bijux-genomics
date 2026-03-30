# `fastq.profile_overrepresented_sequences` corpus-01 method

## Scope
- Stage: `fastq.profile_overrepresented_sequences`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `overrepresented_sequence_fairness`

## Governed tool cohort
- The benchmark runner resolves the tool roster from `bijux-dna registry list-tools --stage fastq.profile_overrepresented_sequences --kind benchmark`.
- The current governed fairness cohort is:
  - `fastqc`
  - `fastq_scan`
  - `seqkit`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Pin the governed overrepresented-sequence contract across the whole corpus:
  - `report_only = true`
  - `mutates_fastq = false`
  - `may_change_read_count = false`
  - `top_k = 50`
  - artifact set:
    - `overrepresented_sequences.tsv`
    - `overrepresented_sequences.json`
    - `overrepresented_report.json`

## Why this stage is benchmarked differently
- `fastq.profile_overrepresented_sequences` does not mutate FASTQ content.
- The benchmark therefore measures runtime plus ranked-sequence stability rather than retention deltas.
- This dossier provides a corpus-wide baseline for comparing observer backends that summarize overrepresented content differently internally but must still satisfy the governed artifact contract.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and ranked-sequence summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or strongest dominant-sequence samples.
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.profile_overrepresented_sequences
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.profile_overrepresented_sequences
```

The default corpus root is loaded from `configs/bench/benchmark.toml`. Update that config or pass `--config` when rerendering against a different governed workspace.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any published report produced from `--dry-run`.
- Reject any published report that omits a tool row for any sample.
- Reject any published report that lacks the governed ranked-sequence artifacts.
- Reject any published report that carries invalid sequence-count, flagged-sequence, or dominant-fraction metrics.

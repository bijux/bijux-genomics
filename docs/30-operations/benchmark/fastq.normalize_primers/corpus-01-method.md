# `fastq.normalize_primers` corpus-01 method

## Scope
- Stage: `fastq.normalize_primers`
- Corpus: `corpus-01`
- Platform target: `apptainer-amd64`
- Benchmark scenario: `primer_normalization_fairness`

## Governed tool cohort
- The benchmark runner must resolve the roster from `bijux-dna registry list-tools --stage fastq.normalize_primers --kind benchmark`.
- The current governed fairness cohort is:
  - `cutadapt`

## Execution contract
- Use normalized FASTQ inputs from `corpus-01/normalized/`.
- Require the balanced corpus contract:
  - `5` ancient single-end
  - `5` ancient paired-end
  - `5` modern single-end
  - `5` modern paired-end
- Hold one governed primer-normalization contract across the full corpus:
  - identical normalized inputs for every backend
  - identical primer-orientation policy
  - identical benchmark scenario id and contract hash in the run manifest
- Preserve backend-native evidence for trimmed or retained primer sequence handling.

## Why `corpus-01` is still useful here
- `corpus-01` is a human DNA cohort rather than an amplicon primer challenge set.
- That makes this benchmark a governed false-positive control as much as a throughput comparison.
- Any future dossier must therefore discuss both runtime and how aggressively each backend claims primer work on this corpus.

## Published artifacts
- `summary.json`: stage-level summary for the corpus run.
- `sample_results.csv`: one row per sample/tool execution.
- `tool_runtime_summary.csv`: per-tool runtime and retention summary.
- `cohort_runtime_summary.csv`: era/layout and size-band breakdowns.
- `sample_runtime_outliers.csv`: slowest or most aggressive samples.
- `benchmark.md`: narrative benchmark dossier for the published corpus run.

## Workflow
```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.normalize_primers
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.normalize_primers
```

The runner resolves the governed corpus root through `configs/bench/benchmark.toml`. Change that config or pass `--config` only when you intentionally target a different governed workspace.

## Guardrails
- Reject any run whose tool roster differs from the governed benchmark cohort.
- Reject any published dossier that omits a tool row for any sample.
- Reject any dossier whose rows drift from the governed primer-normalization contract captured in the run manifest.

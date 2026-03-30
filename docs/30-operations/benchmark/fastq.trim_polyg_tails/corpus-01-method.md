# `fastq.trim_polyg_tails` on `corpus-01`

## Intent

This benchmark measures the governed `fastq.trim_polyg_tails` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

The benchmark contract is:

- full corpus coverage: `20` normalized samples
- balanced cohort coverage: `5` ancient single-end, `5` ancient paired-end, `5` modern single-end, `5` modern paired-end
- full stage tool coverage: `fastp` and `bbduk`
- explicit chemistry contract: `polyx_preset=illumina_twocolor`
- explicit trim threshold: `min_polyg_run=10`

## Workflow

```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.trim_polyg_tails
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.trim_polyg_tails
```

The make targets are thin wrappers over `bijux-dna bench ...` commands. They resolve the governed corpus root and run root from `configs/bench/benchmark.toml`; change that config or pass `--config` only when you intentionally target a different governed workspace.

## Artifact Contract

The runner writes the execution manifest under the Lunarc run root:

- `run_manifest.json`
- `bench/trim_polyg_tails/<sample_id>/report.json`
- `bench/trim_polyg_tails/<sample_id>/bench.jsonl`
- `bench/trim_polyg_tails/<sample_id>/bench.sqlite`

The Rust dossier command publishes the doc set under `docs/30-operations/benchmark/fastq.trim_polyg_tails/corpus-01`:

- `summary.json`
- `sample_results.csv`
- `tool_runtime_summary.csv`
- `cohort_runtime_summary.csv`
- `sample_runtime_outliers.csv`
- `benchmark.md`

## Guardrails

The corpus commands intentionally fail when the run is incomplete or incoherent.

They reject:

- corpus drift from the committed cohort balance
- partial tool rosters
- mixed `polyx_preset` values inside one corpus report
- mixed `min_polyg_run` values inside one corpus report
- missing per-sample tool rows
- backend report format drift between governed tools and native reports

## Interpretation

Use the resulting dossier for:

- comparing runtime cost between `fastp` and `bbduk`
- comparing retained base fractions after governed polyG trimming
- identifying which cohort segments actually exercise polyG cleanup work
- identifying the most expensive or most aggressively trimmed samples

Do not use the benchmark dossier alone to claim:

- biological truth about damage or sequencing chemistry
- generalized performance outside the governed Lunarc platform
- final trimming policy for datasets with a different sequencer chemistry contract

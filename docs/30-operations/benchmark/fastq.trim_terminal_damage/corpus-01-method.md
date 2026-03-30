# `fastq.trim_terminal_damage` corpus-01 method

## Intent

This benchmark measures the governed `fastq.trim_terminal_damage` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform.

The benchmark contract is:

- full corpus coverage: `20` normalized samples
- balanced cohort coverage: `5` ancient single-end, `5` ancient paired-end, `5` modern single-end, `5` modern paired-end
- full stage tool coverage: `adapterremoval`, `cutadapt`, and `seqkit`
- explicit damage policy contract:
  - `damage_mode=ancient`
  - `execution_policy=explicit_terminal_trim`
  - `trim_5p_bases=2`
  - `trim_3p_bases=2`

## Workflow

```bash
cargo run -q -p bijux-dna -- --platform apptainer-amd64 bench corpus-fastq \
  --config configs/bench/benchmark.toml \
  --stage fastq.trim_terminal_damage
cargo run -q -p bijux-dna -- bench corpus-fastq-report \
  --config configs/bench/benchmark.toml \
  --stage fastq.trim_terminal_damage
```

The make targets are thin wrappers over `bijux-dna bench ...` commands. They resolve the governed corpus root and run root from `configs/bench/benchmark.toml`; change that config or pass `--config` only when you intentionally target a different governed workspace.

## Artifact Contract

The runner writes the execution manifest under the Lunarc run root:

- `run_manifest.json`
- `bench/trim_terminal_damage/<sample_id>/report.json`
- `bench/trim_terminal_damage/<sample_id>/bench.jsonl`
- `bench/trim_terminal_damage/<sample_id>/bench.sqlite`

The Rust dossier command publishes the doc set under `docs/30-operations/benchmark/fastq.trim_terminal_damage/corpus-01`:

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
- mixed damage policy rows inside one corpus report
- missing per-sample tool rows
- backend report format drift between governed tools and native reports
- attempts to publish a report from a `--dry-run` manifest

## Interpretation

Use the resulting dossier for:

- comparing runtime cost between the governed terminal-damage backends
- comparing retained base fractions after terminal trimming
- comparing which tools reduce terminal C>T / G>A asymmetry more strongly
- identifying whether modern DNA behaves as a negative-control cohort under an ancient-DNA trim policy

Do not use the benchmark dossier alone to claim:

- final biological damage truth for the cohort
- final preprocessing policy outside the governed Lunarc platform
- that modern cohorts should receive damage-aware trimming in production

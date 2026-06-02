# configs/bench

## What
Configuration knobs for benchmark behavior, independent from benchmark suite data.

## Philosophy
Keep benchmark runtime knobs here while suite definitions live under `crates/bijux-dna-bench/bench/`.

## Knob Categories
- Run counts: cold/warm repetition policy.
- Fairness policy: threads, memory, and tmp isolation rules.
- Runtime constraints: deterministic runner behavior and reproducibility flags.
- Workspace paths: governed local and remote roots for benchmark mirrors, shared cache trees, and private code checkouts.

## Files
- `configs/bench/benchmark.toml`
- `configs/bench/knobs.toml`

## Local Benchmark Readiness Contracts
- `configs/bench/local/tool-families.toml` classifies every governed FASTQ and BAM benchmark tool by its primary benchmark function so readiness reports can group tools consistently across domains.

## Control Plane Contract
- `bijux-dna` is the primary benchmark control plane for workspace lookups, dossier refresh, publication audits, and corpus benchmark execution.
- `bijux-dna bench workspace-value`, `bijux-dna bench corpus-fastq`, `bijux-dna bench corpus-fastq-report`, `bijux-dna bench corpus-fastq-publication-status`, and `bijux-dna bench corpus-fastq-published-dossiers` consume the governed benchmark config directly.
- `bijux-dna bench publication-targets` remains a contract inspection helper that prints governed Rust command lines, not Make targets.
- Top-level scripts under `makes/bin/` should remain thin wrappers or narrow bootstrap utilities, not benchmark orchestration peers to `bijux-dna`.
- `configs/bench/benchmark.toml` is the canonical benchmark contract.
- `BIJUX_BENCHMARK_CONFIG` and shared `--config` CLI options select a different benchmark config when a local or migration workflow needs one.
- Machine-specific roots inside `configs/bench/benchmark.toml` should be supplied through environment placeholders rather than committed absolute paths.

## Workspace Contract
- `[workspace]` in `configs/bench/benchmark.toml` keeps benchmark path policy outside the runners and reporting code.
- `[local].results_root` is the local archive root where mirrored Lunarc artifacts land.
- `[local].cache_mirror_root` is the local path that mirrors the remote shared `.cache` tree under the archive root.
- `[remote].repo_root` is the private frontend checkout used for code sync, not shared benchmark storage.
- `[remote].cache_root`, `results_root`, `extra_data_root`, `containers_root`, and `reference_root` point at governed shared artifacts under the HPC workspace.

## Publication Contract
- `[publication]` in `configs/bench/benchmark.toml` keeps governed corpus publication exclusions outside runner and dossier implementation code.
- `[[publication.corpus_01.contracts]]` rows declare the governed published stage roster, scenario ids, sample scopes, and tool rosters.

## Corpus Contract
- `[corpora.<corpus_id>]` binds each governed benchmark corpus id to the corpus spec consumed by `bijux-dna bench corpus-fastq`.
- `[stage_inputs]` holds benchmark-stage resource bindings such as reference indexes, taxonomy databases, and rRNA bundles so Make wrappers do not become a second source of truth.

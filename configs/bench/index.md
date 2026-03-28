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
- `configs/bench/knobs.toml`
- `configs/bench/publication.toml`
- `configs/bench/workspace.toml`

## Control Plane Contract
- `bijux-dna` is the primary benchmark control plane for workspace lookups, publication target expansion, and corpus benchmark execution.
- `bijux-dna bench workspace-value`, `bijux-dna bench publication-targets`, and `bijux-dna bench corpus-fastq` consume the governed benchmark config directly.
- `makes/bin/benchmark_fastq_corpus/` remains a compatibility and helper package for Python report rendering, audits, and narrow bootstrap utilities.
- Top-level scripts under `makes/bin/` are compatibility entrypoints and should keep shrinking rather than gaining new orchestration logic.
- `configs/bench/workspace.toml` is the single path contract for both Rust and Python benchmark surfaces.
- `BIJUX_FASTQ_CORPUS_CONFIG` and shared `--config` CLI options select a different benchmark config when a local or migration workflow needs one.

## Workspace Contract
- `configs/bench/workspace.toml` keeps benchmark path policy outside the runners and reporting code.
- `[local].results_root` is the local archive root where mirrored Lunarc artifacts land.
- `[local].cache_mirror_root` is the local path that mirrors the remote shared `.cache` tree under the archive root.
- `[remote].repo_root` is the private frontend checkout used for code sync, not shared benchmark storage.
- `[remote].cache_root`, `results_root`, `extra_data_root`, `containers_root`, and `reference_root` point at governed shared artifacts under the HPC workspace.

## Publication Contract
- `configs/bench/publication.toml` keeps governed corpus publication exclusions outside Python support code.
- `[[corpus_01.contracts]]` rows declare the governed published stage roster, scenario ids, sample scopes, and tool rosters.
- `[[corpus_01.exclusions]]` rows declare stage ids and durable reasons for publication exclusions.

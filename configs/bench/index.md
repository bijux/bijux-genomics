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

## Workspace Contract
- `configs/bench/workspace.toml` keeps benchmark path policy outside the runners and reporting code.
- `[local].results_root` is the local archive root where mirrored Lunarc artifacts land.
- `[local].cache_mirror_root` is the local path that mirrors the remote shared `.cache` tree under the archive root.
- `[remote].repo_root` is the private frontend checkout used for code sync, not shared benchmark storage.
- `[remote].cache_root`, `results_root`, `extra_data_root`, `containers_root`, and `reference_root` point at governed shared artifacts under the HPC workspace.

## Publication Contract
- `configs/bench/publication.toml` keeps governed corpus publication exclusions outside Python support code.
- `[[corpus_01.exclusions]]` rows declare stage ids and durable reasons for publication exclusions.

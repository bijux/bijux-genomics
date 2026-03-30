# Benchmark Workspace Contract

## Purpose

This document defines the governed benchmark workspace layout for corpus publication, mirror synchronization, and local report rendering.

`configs/bench/benchmark.toml` is the authority. Benchmark runners, dossier renderers, sync helpers, and audit scripts must read workspace paths from that config instead of hardcoding user paths in code.

`bijux-dna` owns benchmark orchestration and dossier generation against this contract. Make may wrap those Rust commands, but it must not introduce an independent path model or a second execution surface.

Read this together with `docs/30-operations/benchmark/workspace-model.md` for the durable role names used across the benchmark surface.

## Local Workspace

- `workspace.local.results_root` is the stable local archive root where mirrored benchmark artifacts land.
- `workspace.local.cache_mirror_root` preserves the remote shared cache layout inside that archive root.
- `workspace.local.extra_data_root` and `workspace.local.reference_root` point at mirrored non-result assets used by rerenders and audits.

The governed local mirror layout is:

```text
<workspace.local.results_root>/
  corpus_01/<stage_id>/lunarc/...
  <mirrored-remote-cache>/
    results/corpus_01/<stage_id>/lunarc/...
    extra-data/...
    reference/...
```

Use the mirrored remote-cache subtree when publication needs the shared-tree layout exactly as it existed on the frontend. Use the top-level `corpus_01/<stage_id>/lunarc/` archive when a stable local run root is sufficient.

## Remote Workspace

- `workspace.remote.repo_root` is the private frontend checkout used for code sync.
- `workspace.remote.cache_root` is the governed shared benchmark cache root.
- `workspace.remote.results_root` is the canonical shared benchmark results tree.
- `workspace.remote.extra_data_root`, `workspace.remote.containers_root`, and `workspace.remote.reference_root` are the shared non-result benchmark assets.
- `workspace.layout.stage_runs.*` defines the relative templates for remote results roots, local cache-mirror results roots, and local archive results roots.

The code checkout and the shared cache tree are separate contracts. Repo sync belongs under `workspace.remote.repo_root`. Benchmark artifacts belong under the shared cache layout rooted at `workspace.remote.cache_root`.

## Sync Defaults

- `workspace.sync.defaults.pull_base` is the governed local base path used when a benchmark pull does not receive an explicit destination.
- `workspace.sync.defaults.pull_mode` is the default sync mode for the benchmark pull surface.
- `workspace.sync.defaults.include_profile` and `workspace.sync.defaults.exclude_profile` are the default sync profiles for benchmark pulls.
- `workspace.sync.defaults.clean_context` and `workspace.sync.defaults.allow_dirty` define the repo-sync safety posture for benchmark pushes.
- `workspace.sync.defaults.include_containers_manifest` and `workspace.sync.defaults.data_manifest_glob` define supplemental artifacts mirrored alongside results.

Make targets and Rust sync commands should all load these defaults from `configs/bench/benchmark.toml` before consulting environment overrides.

The governed override surface is:

- `BIJUX_BENCHMARK_CONFIG` for environment-driven benchmark commands
- shared `--config` CLI options for `bijux-dna bench ...`

## Publication Rules

- Published FASTQ dossiers should resolve default run roots from `workspace.remote.results_root` and local mirror roots from `workspace.local.cache_mirror_root`.
- Extra-data defaults should resolve from `workspace.remote.extra_data_root` for shared runs and from `workspace.local.extra_data_root` for local mirrors.
- Reference defaults should resolve from `workspace.remote.reference_root` for shared runs and from `workspace.local.reference_root` for local mirrors.
- A benchmark helper should never infer authoritative roots from `corpus_root.parent` or from a guessed `.cache` segment when the workspace contract already names the path.

## Review Checklist

- If a benchmark helper needs a path, add it to `configs/bench/benchmark.toml` before embedding a formula in code.
- If a benchmark workflow needs new orchestration, add it to `bijux-dna` before adding another wrapper layer.
- If a benchmark artifact is mirrored locally, keep it under `local.results_root` with the governed tree shape above.
- If a path appears in generated docs, prefer the configured workspace contract over historical private-path aliases.
- If a publication refresh is complete, the checked-in ledger set should agree across `corpus-01-status.*`, `corpus-01-results-status.*`, `corpus-01-dossier-index.*`, and `corpus-01-remediation-queue.*`.

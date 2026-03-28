# Benchmark Workspace Contract

## Purpose

This document defines the governed benchmark workspace layout for corpus publication, mirror synchronization, and local report rendering.

`configs/bench/workspace.toml` is the authority. Benchmark runners, renderers, sync helpers, and audit scripts must read workspace paths from that config instead of hardcoding user paths in code.

`bijux-dna` should own benchmark orchestration against this contract. Python under `makes/bin/benchmark_fastq_corpus/` is now a compatibility and helper layer, not the intended primary execution surface.

Read this together with `docs/benchmark/workspace-model.md` for the durable role names used across the benchmark surface.

## Local Workspace

- `local.results_root` is the stable local archive root where mirrored benchmark artifacts land.
- `local.cache_mirror_root` is the local path that mirrors the remote shared `.cache` tree under the archive root.
- `local.extra_data_root` and `local.reference_root` name the local mirrored non-result asset roots directly.

The governed local mirror layout is:

```text
<local.results_root>/
  corpus_01/<stage_id>/lunarc/...
  home/bijan/lu2024-12-24/.cache/
    results/corpus_01/<stage_id>/lunarc/...
    extra-data/...
    reference/...
```

Use the `home/.../.cache` mirror when publication needs the shared-tree layout exactly as it existed on the frontend. Use the top-level `corpus_01/<stage_id>/lunarc/` archive when a stable local run root is sufficient.

## Remote Workspace

- `remote.repo_root` is the private frontend checkout used for code sync.
- `remote.cache_root` is the governed shared benchmark cache root.
- `remote.results_root` is the canonical shared benchmark results tree.
- `remote.results_legacy_root` is the legacy shared results root kept only for migration and audit compatibility.
- `remote.extra_data_root`, `remote.containers_root`, and `remote.reference_root` are the shared non-result benchmark assets.
- `layout.stage_runs.*` defines the governed relative templates for remote results roots, local cache-mirror results roots, and local archive results roots.

The code checkout and the shared cache tree are separate contracts. Repo sync belongs under `remote.repo_root`. Benchmark artifacts belong under the shared cache layout rooted at `remote.cache_root`.

## Sync Defaults

- `sync.defaults.pull_base` is the governed local base path used when a benchmark pull does not receive an explicit destination.
- `sync.defaults.pull_mode` is the default sync mode for the benchmark pull surface.
- `sync.defaults.include_profile` and `sync.defaults.exclude_profile` are the governed default sync profiles for benchmark pulls.
- `sync.defaults.clean_context` and `sync.defaults.allow_dirty` define the default repo-sync safety posture for benchmark pushes.
- `sync.defaults.include_containers_manifest` and `sync.defaults.data_manifest_glob` define the default supplemental artifacts mirrored alongside results.

Make targets, Python tooling, and Rust sync commands should all load these defaults from `configs/bench/workspace.toml` before consulting environment overrides.

The governed override surface is:

- `BIJUX_FASTQ_CORPUS_CONFIG` for make-driven and environment-driven Python calls
- shared `--config` CLI options for `bijux-dna bench corpus-fastq` and compatibility Python entrypoints

## Publication Rules

- Published FASTQ dossiers should resolve default run roots from `remote.results_root` and local mirror roots from `local.cache_mirror_root`.
- Extra-data defaults should resolve from `remote.extra_data_root` for shared runs and from `local.extra_data_root` for local mirrors.
- Reference defaults should resolve from `remote.reference_root` for shared runs and from `local.reference_root` for local mirrors.
- A benchmark helper should never infer authoritative roots from `corpus_root.parent` or from a guessed `.cache` segment when the workspace contract already names the path.

## Review Checklist

- If a benchmark helper needs a path, add it to `configs/bench/workspace.toml` before embedding a formula in code.
- If a benchmark workflow needs new orchestration, add it to `bijux-dna` before creating another Python runner.
- If a benchmark artifact is mirrored locally, keep it under `local.results_root` with the governed tree shape above.
- If a path appears in generated docs, prefer the configured workspace contract over historical private-path aliases.
- If a publication refresh is complete, the checked-in ledger set should agree across `corpus-01-status.*`, `corpus-01-results-status.*`, `corpus-01-dossier-index.*`, and `corpus-01-remediation-queue.*`.

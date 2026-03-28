# Benchmark Workspace Model

## Purpose

This document names the durable benchmark workspace roles used across sync, publication, and audit tooling.

Use these terms consistently in code, docs, and commit messages:

- `private frontend repo`: the remote code checkout named by `remote.repo_root`
- `shared benchmark cache`: the remote shared artifact tree rooted at `remote.cache_root`
- `local benchmark archive`: the local artifact workspace rooted at `local.results_root`
- `local cache mirror`: the local path named by `local.cache_mirror_root` that mirrors the remote shared cache layout

`configs/bench/workspace.toml` is the authority for every root in this model.

## Root Roles

### Private Frontend Repo

- Purpose: code sync, submitted jobs, generated manifests that belong to the repo checkout
- Contract root: `remote.repo_root`
- Storage rule: never treat this tree as shared benchmark storage

### Shared Benchmark Cache

- Purpose: shared results, extra-data, reference assets, and container assets used by governed benchmark runs
- Contract root: `remote.cache_root`
- Child roots:
  - `remote.results_root`
  - `remote.results_legacy_root`
  - `remote.extra_data_root`
  - `remote.containers_root`
  - `remote.reference_root`

### Local Benchmark Archive

- Purpose: durable local mirror used by renderers, repair tools, and audits
- Contract root: `local.results_root`
- Storage rule: keep mirrored benchmark artifacts here rather than ad hoc download directories

### Local Cache Mirror

- Purpose: preserve the remote shared-cache tree shape so localized report paths and extra-data references still resolve
- Contract root: `local.cache_mirror_root`

## Governed Layout

```text
remote.repo_root/
  bijux-dna checkout

remote.cache_root/
  results/
    corpus_01/<stage_id>/lunarc/...
  extra-data/
    benchmark/...
  reference/
    benchmark/...
  bijux-dna-container/
    apptainer/...

local.results_root/
  corpus_01/<stage_id>/lunarc/...
  home/bijan/lu2024-12-24/.cache/
    results/corpus_01/<stage_id>/lunarc/...
    extra-data/benchmark/...
    reference/benchmark/...
```

## Publication Rules

- Repo sync targets the private frontend repo only.
- Benchmark artifact sync targets the shared benchmark cache only.
- Published dossiers should record the run root they used and audits should identify both the selected run root and the newest mirrored alternative.
- Extra-data dependencies required for publication belong in governed sync profiles and governed refresh commands.

## Naming Rules

- Prefer role names such as `shared benchmark cache` and `local benchmark archive` over site-specific shorthand.
- Treat site identifiers such as `lunarc` as execution provenance, not as the workspace model itself.
- When a path contract changes, update `configs/bench/workspace.toml` and the benchmark docs together.

# Benchmark Workspace Model

## Purpose

This document names the durable benchmark workspace roles used across sync, publication, and audit tooling.

Use these terms consistently in code, docs, and commit messages:

- `private frontend repo`: the remote code checkout named by `workspace.remote.repo_root`
- `shared benchmark cache`: the remote shared artifact tree rooted at `workspace.remote.cache_root`
- `local benchmark archive`: the local artifact workspace rooted at `workspace.local.results_root`
- `local cache mirror`: the local path named by `workspace.local.cache_mirror_root` that mirrors the remote shared cache layout

`configs/bench/benchmark.toml` is the authority for every root in this model.
`bijux-dna` consumes that authority directly, and Make wrappers must preserve that ownership rather than restating benchmark path rules.

## Root Roles

### Private Frontend Repo

- Purpose: code sync, submitted jobs, generated manifests that belong to the repo checkout
- Contract root: `workspace.remote.repo_root`
- Storage rule: never treat this tree as shared benchmark storage

### Shared Benchmark Cache

- Purpose: shared results, extra-data, reference assets, and container assets used by governed benchmark runs
- Contract root: `workspace.remote.cache_root`
- Child roots:
  - `workspace.remote.results_root`
  - `workspace.remote.extra_data_root`
  - `workspace.remote.containers_root`
  - `workspace.remote.reference_root`

### Local Benchmark Archive

- Purpose: durable local mirror used by renderers, repair tools, and audits
- Contract root: `workspace.local.results_root`
- Storage rule: keep mirrored benchmark artifacts here rather than ad hoc download directories

### Local Cache Mirror

- Purpose: preserve the remote shared-cache tree shape so localized report paths and extra-data references still resolve
- Contract root: `workspace.local.cache_mirror_root`

## Governed Layout

```text
workspace.remote.repo_root/
  bijux-dna checkout

workspace.remote.cache_root/
  results/
    corpus_01/<stage_id>/lunarc/...
  extra-data/
    benchmark/...
  reference/
    benchmark/...
  bijux-dna-container/
    apptainer/...

workspace.local.results_root/
  corpus_01/<stage_id>/lunarc/...
  <mirrored-remote-cache>/
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
- Prefer portable platform ids such as `apptainer-amd64` in new docs and scripts; keep site aliases such as `lunarc-apptainer` only for backward compatibility with existing manifests and operator workflows.
- When a path contract changes, update `configs/bench/benchmark.toml` and the benchmark docs together.

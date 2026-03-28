# Benchmark Workflow Operations

## Purpose

This document covers two operational contracts:

- how to mirror the governed shared `.cache` tree into the local benchmark archive workspace
- how to move the benchmark workflow to another cluster by updating configuration rather than rewriting benchmark code

Read this together with `docs/benchmark/workspace-contract.md` and `configs/bench/workspace.toml`.

## Mirror The Shared Cache Tree

1. Sync the private benchmark repo checkout to `remote.repo_root`.
2. Sync the governed shared cache tree rooted at `remote.cache_root`.
3. Mirror the pulled cache tree under `local.results_root` so the shared path appears at `local.cache_mirror_root`.
4. Keep canonical local stage archives under `local.results_root/corpus_01/<stage_id>/lunarc/`.
5. Use the `home/.../.cache` mirror when publication or artifact localization needs the original shared-tree layout.

The stable local mirror contract is:

```text
<local.results_root>/
  corpus_01/<stage_id>/lunarc/...
  home/bijan/lu2024-12-24/.cache/
    results/corpus_01/<stage_id>/lunarc/...
    extra-data/...
    reference/...
```

Do not scatter benchmark pulls across ad hoc local directories. Keep mirrored artifacts under `local.results_root` so renderers, audits, and repair tools can resolve the same path contract.

## Move To Another Cluster With Config

1. Copy `configs/bench/workspace.toml` and update the `[remote]` paths for the new frontend checkout, shared cache root, results root, extra-data root, container root, and reference root.
2. Update `[local]` paths only if the local archive workspace changes.
3. Update any cluster-specific sync profile under `configs/hpc/` so repo sync and cache sync target the new remote roots.
4. Keep benchmark runners and report renderers unchanged unless the new cluster requires a genuinely different execution contract.
5. Re-run the benchmark contract tests before publishing refreshed dossiers.

The benchmark Python support layer should consume `configs/bench/workspace.toml` rather than embedding cluster-specific paths in code. A cluster migration is complete only when the config changes are sufficient and the benchmark suite still passes without path edits in `makes/bin`.

## Publication Checklist

- Confirm `configs/bench/workspace.toml` names the correct `local` and `remote` roots.
- Confirm the local mirror under `local.cache_mirror_root` contains the required `results`, `extra-data`, and `reference` trees.
- Refresh corpus dossiers from the governed report targets.
- Re-run the publication audit after the refresh.
- Review `docs/benchmark/corpus-01-dossier-index.json` and `docs/benchmark/corpus-01-dossier-index.md` to confirm each dossier freshness stamp and published run-root source.
- Review `docs/benchmark/corpus-01-results-status.json` and `docs/benchmark/corpus-01-results-status.md` to confirm the local mirror still matches the published summaries.
- Review `docs/benchmark/corpus-01-remediation-queue.json` and `docs/benchmark/corpus-01-remediation-queue.md` to confirm the remaining open stages, recommended action, and queue owner.

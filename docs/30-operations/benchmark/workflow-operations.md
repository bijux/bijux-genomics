# Benchmark Workflow Operations

## Purpose

This document covers two operational contracts:

- how to mirror the governed shared `.cache` tree into the local benchmark archive workspace
- how to move the benchmark workflow to another cluster by updating configuration rather than rewriting benchmark code

Read this together with `docs/30-operations/benchmark/workspace-model.md`, `docs/30-operations/benchmark/workspace-contract.md`, and `configs/bench/benchmark.toml`.

Repo sync belongs on the private frontend home. Benchmark artifacts belong on the shared cache contract. Keep those responsibilities separate in code, automation, and operator docs.

`bijux-dna` is the benchmark execution surface. New benchmark orchestration and corpus dossier generation should land under the Rust CLI, especially `bijux-dna bench corpus-fastq` and `bijux-dna bench corpus-fastq-report`, rather than in new helper layers under `makes/bin/`.

## Encrypted Slurm Bundle Workflow

Campaign submissions now emit encrypted `.results` and `.code` bundles together with sidecar files.

- Keep `security.encrypt_operator_outputs = false` by default so operators can inspect `.log/.out/.err`.
- Use `bijux-dna slurm copy-back-manifest` to capture bundle and sidecar paths for local import.
- Use `bijux-dna slurm verify-bundle` before decrypting copied artifacts.
- Use `bijux-dna slurm decrypt-bundle --out-dir artifacts/investigation/decrypt` for local review.
- Use `bijux-dna slurm import-replay` for one results/code pair and inspect
  `import-replay-report.json`.
- Use `bijux-dna slurm import-campaign` for bulk copied campaign trees and inspect
  `import-campaign-report.json`.
- Use `bijux-dna slurm export-failure-bundle` when a single benchmark row must be shared for debug.
- Use `bijux-dna slurm share-bundle` with a profile under `configs/hpc/campaign/sharing/`.

## Mirror The Shared Cache Tree

1. Sync the private benchmark repo checkout to `workspace.remote.repo_root`.
2. Sync the governed shared cache tree rooted at `workspace.remote.cache_root`.
3. Mirror the pulled cache tree under `workspace.local.results_root` so the shared path appears at `workspace.local.cache_mirror_root`.
4. Keep canonical local stage archives under `workspace.local.results_root/corpus_01/<stage_id>/lunarc/`.
5. Use the mirrored remote-cache subtree when publication or artifact localization needs the original shared-tree layout.

The stable local mirror contract is:

```text
<workspace.local.results_root>/
  corpus_01/<stage_id>/lunarc/...
  <mirrored-remote-cache>/
    results/corpus_01/<stage_id>/lunarc/...
    extra-data/...
    reference/...
```

Do not scatter benchmark pulls across ad hoc local directories. Keep mirrored artifacts under `workspace.local.results_root` so renderers, audits, and repair tools can resolve the same path contract.

The governed publication refresh sequence is:

```bash
BENCHMARK_SYNC_INCLUDE_CONTAINERS_MANIFEST=1 \
BENCHMARK_SYNC_DATA_MANIFEST_GLOB="benchmark/fastq.screen_taxonomy/read_screening/read_screening/taxonomy_db/lineage.tsv" \
cargo run -q -p bijux-dna-dev -- hpc run lunarc/pull \
  --include-profile pull-benchmark-publication \
  --exclude-profile pull-full-default
cargo run -q -p bijux-dna -- bench normalize-workspace-layout \
  --config configs/bench/benchmark.toml \
  --confirm
cargo run -q -p bijux-dna -- bench corpus-fastq-published-dossiers \
  --config configs/bench/benchmark.toml
```

Those commands pull the governed results mirror, preserve the sync profile contract from `configs/hpc/benchmark_sync_profiles.toml`, normalize the local archive layout, refresh the published dossiers, and rebuild the publication audits.

`make benchmark-publication-refresh` remains an optional wrapper around that governed sequence; it is not the benchmark authority. `make benchmark-lunarc-publication-refresh` remains only as a compatibility alias.

The default pull base, pull mode, sync profiles, repo cleanliness checks, and supplemental manifest settings now belong to `[sync.defaults]` in `configs/bench/benchmark.toml`. Use environment overrides only when the current operation genuinely needs to diverge from the governed defaults.

When a caller needs a non-default benchmark workspace contract, set `BIJUX_BENCHMARK_CONFIG` or pass `--config <path>` to `bijux-dna bench ...`. Do not fork path formulas into new scripts.

When driving the sync surface directly through `cargo run -p bijux-dna-dev -- hpc run ...`, use `benchmark-sync-pull` and `benchmark-sync-push`.

## Move To Another Cluster With Config

1. Copy `configs/bench/benchmark.toml` and update the `[workspace.remote]` roots for the new frontend checkout, shared cache root, results root, extra-data root, container root, and reference root.
2. Update `[workspace.local]` roots only if the local archive workspace changes.
3. Update any cluster-specific sync profile under `configs/hpc/` so repo sync and cache sync target the new remote roots.
4. Keep benchmark runners and dossier generators unchanged unless the new cluster requires a genuinely different execution contract.
5. Re-run the benchmark contract tests before publishing refreshed dossiers.

The benchmark control plane should consume `configs/bench/benchmark.toml` through `bijux-dna` rather than embedding cluster-specific paths in code. A cluster migration is complete only when the config changes are sufficient and the benchmark suite still passes without path edits in Make wrappers, docs, or Rust benchmark orchestration code.

## Sync Profile Contract

`configs/hpc/benchmark_sync_profiles.toml` is the benchmark sync profile registry. Profiles must describe more than rsync include and exclude files:

- `workspace_scope` declares the governed benchmark workflow that owns the profile.
- `pull_destination` names the workspace destination contract instead of relying on an operator-specific shell default.
- `remote_roots` lists the governed remote roots the profile is allowed to mirror.
- `data_manifest_globs` records extra-data dependencies needed to render or audit dossiers after the pull.

The `pull-benchmark-publication` profile is the governed profile for corpus-01 FASTQ dossier publication. It mirrors the shared results trees plus the taxonomy lineage file needed by `fastq.screen_taxonomy`.

If you need environment overrides, use the `BENCHMARK_SYNC_*` variables. The workspace contract in `configs/bench/benchmark.toml`, including `[sync.defaults]`, should still remain the primary source of truth.

The repo push marker `BENCHMARK_SYNC_SOURCE.json` should carry the benchmark workspace roots alongside the synced commit so the remote checkout records which benchmark environment contract the push was prepared against.

## Publication Checklist

- Confirm `configs/bench/benchmark.toml` names the correct `workspace.local` and `workspace.remote` roots.
- Confirm the local mirror under `workspace.local.cache_mirror_root` contains the required `results`, `extra-data`, and `reference` trees.
- Run the governed sync plus Rust publication refresh sequence above when the source run lives on Lunarc and publication inputs need a fresh sync.
- Refresh corpus dossiers from the governed report targets when a local rerender is sufficient.
- Re-run the publication audit after the refresh if you did not use the governed one-shot command.
- Review `docs/30-operations/benchmark/corpus-01-dossier-index.json` and `docs/30-operations/benchmark/corpus-01-dossier-index.md` to confirm each dossier freshness stamp and published run-root source.
- Review `docs/30-operations/benchmark/workspace-layout-status.json` and `docs/30-operations/benchmark/workspace-layout-status.md` to confirm the mirrored benchmark archive still uses the governed root layout.
- Review `docs/30-operations/benchmark/corpus-01-results-status.json` and `docs/30-operations/benchmark/corpus-01-results-status.md` to confirm the local mirror still matches the published summaries.
- Review `docs/30-operations/benchmark/corpus-01-remediation-queue.json` and `docs/30-operations/benchmark/corpus-01-remediation-queue.md` to confirm the remaining open stages, recommended action, and queue owner.

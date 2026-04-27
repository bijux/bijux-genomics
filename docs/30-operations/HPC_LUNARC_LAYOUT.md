# HPC Lunarc Layout

This document describes the Lunarc benchmark workspace contract as configured through
[configs/bench/benchmark.toml](../../configs/bench/benchmark.toml). Read it together with
[benchmark/workspace-contract.md](benchmark/workspace-contract.md) and
[benchmark/workspace-model.md](benchmark/workspace-model.md).

## Roots

- `workspace.remote.repo_root`: private frontend checkout used for repo sync and operator commands
- `workspace.remote.corpus_root`: governed benchmark corpus checkout
- `workspace.remote.results_root`: governed shared results tree
- `workspace.remote.containers_root`: governed shared container asset root
- `workspace.local.results_root`: local benchmark archive used for mirrored artifacts and publication work

## Invariants

1. `bijux-dna` owns benchmark orchestration and corpus dossier generation; Make should stay a thin wrapper over Rust commands.
2. Repo sync and benchmark artifact sync are separate responsibilities. Code belongs under `workspace.remote.repo_root`; shared artifacts belong under the configured workspace roots.
3. Corpus benchmarks should resolve their inputs through
   [configs/bench/benchmark.toml](../../configs/bench/benchmark.toml), not through hardcoded
   frontend paths in scripts, docs, or wrapper targets.
4. Every HPC run must carry reproducibility metadata and run-context metadata in
   [RUN_ARTIFACTS.md](RUN_ARTIFACTS.md).
5. Result paths remain run-scoped and timestamped according to the configured layout templates.
6. Shared temp directories are forbidden. Each run must use its own run-scoped temp path.
7. Pulled Lunarc artifacts must land under `workspace.local.results_root` so publication, audits, and rerenders share one local contract.

## Commands

- Validate the configured benchmark contract:
  - `cargo run -q -p bijux-dna -- bench config validate --config configs/bench/benchmark.toml`
- Validate HPC status:
  - `bijux status --hpc`
- Pull Lunarc results into the local mirror:
  - `cargo run -q -p bijux-dna-dev -- hpc run lunarc/pull --include-profile pull-results-default --exclude-profile pull-full-default`
- Pull results and clear the remote results payload:
  - `cargo run -q -p bijux-dna-dev -- hpc run lunarc/pull --include-profile pull-results-default --exclude-profile pull-full-default`
  - Resolve `workspace.remote.ssh_host` and `workspace.remote.results_root` through `cargo run -q -p bijux-dna -- bench workspace-value --config configs/bench/benchmark.toml <key>`, then prune that remote results root while preserving `site_lock.json`.
- Audit the frontend footprint:
  - `make lunarc-footprint`
- Clear transient build residue from the frontend repo checkout:
  - `make lunarc-prune-code`

`make benchmark-sync-pull-results` and `make benchmark-sync-pull-results-prune` are the canonical optional wrappers around the governed sync contract above. `make pull-lunarc-results` and `make pull-lunarc-results-prune` remain compatibility aliases.

## What
Defines the Lunarc-facing workspace roles and operational invariants for deterministic benchmark execution.

## Why
Separating repo sync, corpus inputs, shared artifacts, and local mirrors avoids cross-contamination and keeps cluster migration in configuration instead of code edits.

## Non-goals
- Describe non-Lunarc cluster layouts.
- Replace per-pipeline runbooks.

## Contracts
- Runs use only the configured benchmark workspace roots.
- Reproducibility metadata is written for each run.
- Result paths remain run-scoped and timestamped.

## Examples
- `cargo run -q -p bijux-dna -- bench config validate --config configs/bench/benchmark.toml`
- `bijux status --hpc`

## Failure modes
- Missing one of the required roots causes profile validation failures.
- Shared temp/output paths can break reproducibility guarantees.

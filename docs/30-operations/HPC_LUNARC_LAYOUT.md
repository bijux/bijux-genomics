# HPC Lunarc Layout

This document describes the Lunarc benchmark workspace contract as configured through `configs/bench/benchmark.toml`.

## Roots

- `workspace.remote.repo_root`: private frontend checkout used for repo sync and operator commands
- `workspace.remote.corpus_root`: governed benchmark corpus checkout
- `workspace.remote.results_root`: governed shared results tree
- `workspace.remote.containers_root`: governed shared container asset root
- `workspace.local.results_root`: local benchmark archive used for mirrored artifacts and publication work

## Invariants

1. `bijux-dna` owns benchmark orchestration; Make should stay a thin wrapper over Rust commands.
2. Repo sync and benchmark artifact sync are separate responsibilities. Code belongs under `workspace.remote.repo_root`; shared artifacts belong under the configured workspace roots.
3. Corpus benchmarks should resolve their inputs through `configs/bench/benchmark.toml`, not through hardcoded frontend paths in scripts or docs.
4. Every HPC run must carry reproducibility metadata and run-context metadata in `run_manifest.json`.
5. Result paths remain run-scoped and timestamped according to the configured layout templates.
6. Shared temp directories are forbidden. Each run must use its own run-scoped temp path.
7. Pulled Lunarc artifacts must land under `workspace.local.results_root` so publication, audits, and rerenders share one local contract.

## Commands

- Validate the configured benchmark contract:
  - `cargo run -q -p bijux-dna -- bench config validate --config configs/bench/benchmark.toml`
- Validate HPC status:
  - `bijux status --hpc`
- Pull Lunarc results into the local mirror:
  - `make pull-lunarc-results`
- Pull results and clear the remote results payload:
  - `make pull-lunarc-results-prune`
- Audit the frontend footprint:
  - `make lunarc-footprint`
- Clear transient build residue from the frontend repo checkout:
  - `make lunarc-prune-code`

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

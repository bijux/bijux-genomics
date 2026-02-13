# REPRODUCIBILITY

Owner: Operations
Scope: Offline and deterministic run reproduction
Last reviewed: 2026-02-11
Contract version: v1
Applies to crates: bijux-dna-runtime, bijux-dna-runner, bijux-dna-pipelines

## What
Defines how to reproduce pipeline runs deterministically from manifests, locked tool digests, and pinned params.

## Why
Ensures scientific and engineering results are re-runnable and reviewable across environments.

## Non-goals
- Describing benchmark objective tuning.
- Explaining stage-level biology.

## Contracts
- `run_manifest.json` and `run_manifest.lock.json` must be emitted.
- Tool digests in lock artifacts must be immutable.
- Param hash changes must change profile hash snapshots.

## Examples
- Re-run with the same manifest and lock on an offline worker.
- Verify `manifest_signature_sha256` equality before comparing metrics.
- Verify benchmark suite/config locations with `bijux dna bench status`.
- Benchmark suites are owned under `crates/bijux-dna-bench/bench/suites/`.
- Refresh toy inputs deterministically with `make refresh-toy` (writes `assets/toy/core-v1/`).
- Refresh toy-run golden outputs deterministically with `make refresh-golden` (writes `assets/golden/toy-runs-v1/`).

## Failure modes
- Floating tool tags or mutable pins break reproducibility.
- Missing lock artifacts prevent deterministic replay.

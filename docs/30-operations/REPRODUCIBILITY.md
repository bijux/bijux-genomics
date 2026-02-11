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

## Failure modes
- Floating tool tags or mutable pins break reproducibility.
- Missing lock artifacts prevent deterministic replay.

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

## Purpose
Define reproducibility guarantees and validation anchors for production and reference runs.

## Scope
Applies to manifest/lock/hash-based replay and contract-level reproducibility checks.

## Non-goals
- Describing benchmark objective tuning.
- Explaining stage-level biology.

## Contracts
- [RUN_ARTIFACTS.md](RUN_ARTIFACTS.md) defines the required `run_manifest.json` and
  `run_manifest.lock.json` outputs.
- Tool digests in lock artifacts must be immutable.
- Param hash changes must change profile hash snapshots.
- Reference panels are locked inputs defined under
  [configs/vcf/panels/panels.toml](../../configs/vcf/panels/panels.toml).
- Panel catalog entries must include pinned `version`, pinned `url`, and `checksum_sha256`.
- Floating URLs are forbidden for panel artifacts.
- Panel metadata must include `population_set`, `genome_build`, and `variant_set_compatibility` to prevent ancestry/build mismatches.
- Canonical lock outputs are
  [configs/vcf/panels/locks/lock.json](../../configs/vcf/panels/locks/lock.json) and
  [configs/vcf/panels/locks/lock.json.sha256](../../configs/vcf/panels/locks/lock.json.sha256).

## HPC Forward-compat
- Enabling HPC profile changes physical data/container/output roots, not contract semantics.
- Reproducibility comparison must use manifest/lock/hash equality rather than absolute filesystem paths.
- Site-managed container caches are valid when digest pins remain unchanged.

## Examples
- Re-run with the same manifest and lock on an offline worker.
- Verify `manifest_signature_sha256` equality before comparing metrics.
- Verify benchmark suite/config locations with `bijux-dna bench status`.
- Benchmark suites are owned under
  [crates/bijux-dna-bench/bench/suites/](../../crates/bijux-dna-bench/bench/suites/).
- Refresh toy inputs deterministically with `make refresh-toy` (writes `assets/toy/core-v1/`).
- Refresh toy-run golden outputs deterministically with `make refresh-golden` (writes `assets/golden/toy-runs-v1/`).

## Failure modes
- Floating tool tags or mutable pins break reproducibility.
- Missing lock artifacts prevent deterministic replay.

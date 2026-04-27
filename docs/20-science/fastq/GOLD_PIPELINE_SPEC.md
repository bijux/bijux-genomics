# FASTQ Gold Pipeline Spec

## What
Defines gold‑standard FASTQ pipeline expectations.

## Why
Provides a reference baseline for audits.

## Non-goals
- Performance benchmarking.

## Contracts
- Pipeline stage inventory and governed FASTQ stage IDs live in
  [domain/fastq/index.yaml](../../../domain/fastq/index.yaml).
- Default stage ordering and route families live in
  [domain/fastq/route_policies.toml](../../../domain/fastq/route_policies.toml).
- Pinned default tools and profile semantics live in
  [domain/fastq/docs/DEFAULT_SETTINGS.md](../../../domain/fastq/docs/DEFAULT_SETTINGS.md).

## Examples
- Default pipeline profile used for regression checks.

## Failure modes
- Drift from gold defaults requires explicit approval.

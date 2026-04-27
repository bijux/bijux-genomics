# Refusals

## What
Explicit statements of what Bijux Genomics does not support.

## Why
Refusals prevent accidental scope creep and keep policies enforceable.

## Non-goals
- Supporting ad‑hoc shell execution in stages.
- Accepting untyped identifiers in contracts.

## Contracts
- [../10-architecture/BOUNDARY_MAP.md](../10-architecture/BOUNDARY_MAP.md)
- [../../domain/fastq/route_policies.toml](../../domain/fastq/route_policies.toml)

## Examples
- Stages never build command lines.

## Failure modes
- Attempts to bypass contracts are rejected by policy tests.

## Stage Refusals
- `fastq.infer_asvs`
  - route_scope: `shotgun_standard`, `shotgun_adna`, `host_associated_metagenome`
  - status: refused for governed shotgun-family routes
  - authority: [../../domain/fastq/route_policies.toml](../../domain/fastq/route_policies.toml)
  - context: [../20-science/fastq/STAGE_CATALOG.md](../20-science/fastq/STAGE_CATALOG.md)

# SSOT

Owner: Architecture
Scope: Contract and boundary authority
Last reviewed: 2026-02-07
Contract version: v1
Applies to crates: bijux-dna-core, bijux-dna-engine, bijux-dna-runtime, bijux-dna-runner, bijux-dna-api

## Purpose
Define single-source-of-truth boundaries for IDs, stage specs, metrics, and generated configs.

## Scope
Applies to repository-wide ownership and consumption boundaries for typed contracts.

## What
Single Source of Truth for IDs, stage specs, tool selection, and metrics definitions.
Authority rule: `domain/*/**/*.yaml` is the authored source of truth for domain stage/tool contracts.

## Why
Prevents duplicated semantics and inconsistent identifiers.

## Non-goals
- Duplicating IDs in multiple crates.

## Contracts
- ID types in `bijux-dna-core`.
- Stage specs in `bijux-dna-stages-*`.
- Metrics definitions in domain crates.
- Domain metadata is authored in `domain/**`; generated config views are produced by `bijux-dna-domain-compiler`.
- Domain versioning is explicit in `domain/*/index.yaml` with `domain_version: v1|v2`.
- VCF downstream policy baseline is `domain_version: v2`.
- Generated config scope is fixed to:
  - `configs/ci/registry/tool_registry.toml`
  - `configs/ci/stages/stages.toml`
  - `configs/ci/tools/images.toml`
- Generated files must carry `GENERATED - DO NOT EDIT` headers with source commit hash and are verified in CI.

## Examples
- `StageId` is a typed ID defined only in core.

## Failure modes
- Literal IDs outside core fail policy scans.

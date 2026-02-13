# SSOT

Owner: Architecture
Scope: Contract and boundary authority
Last reviewed: 2026-02-07
Contract version: v1
Applies to crates: bijux-dna-core, bijux-dna-engine, bijux-dna-runtime, bijux-dna-runner, bijux-dna-api

## What
Single Source of Truth for IDs, stage specs, tool selection, and metrics definitions.

## Why
Prevents duplicated semantics and inconsistent identifiers.

## Non-goals
- Duplicating IDs in multiple crates.

## Contracts
- ID types in `bijux-dna-core`.
- Stage specs in `bijux-dna-stages-*`.
- Metrics definitions in domain crates.
- Domain metadata is authored in `domain/**`; generated config views are produced by `bijux-dna-domain-compiler`.
- Generated config scope is fixed to:
  - `configs/ci/tool_registry.toml`
  - `configs/ci/stages.toml`
  - `configs/ci/images.toml`
- Generated files must carry `GENERATED - DO NOT EDIT` headers with source commit hash and are verified in CI.

## Examples
- `StageId` is a typed ID defined only in core.

## Failure modes
- Literal IDs outside core fail policy scans.

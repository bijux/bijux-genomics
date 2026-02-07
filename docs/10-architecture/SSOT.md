# SSOT

Owner: Architecture
Scope: Contract and boundary authority
Last reviewed: 2026-02-07
Contract version: v1
Applies to crates: bijux-core, bijux-engine, bijux-runtime, bijux-runner, bijux-api

## What
Single Source of Truth for IDs, stage specs, tool selection, and metrics definitions.

## Why
Prevents duplicated semantics and inconsistent identifiers.

## Non-goals
- Duplicating IDs in multiple crates.

## Contracts
- ID types in `bijux-core`.
- Stage specs in `bijux-stages-*`.
- Metrics definitions in domain crates.

## Examples
- `StageId` is a typed ID defined only in core.

## Failure modes
- Literal IDs outside core fail policy scans.

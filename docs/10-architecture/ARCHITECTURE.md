# Architecture

## What:
Defines the repository architecture rule for SSOT ownership and consumption boundaries.

## Why:
Prevents drift between authored domain data, generated configs, and runtime/planner behavior.

## Non-goals:
- Defining crate-local implementation details.
- Duplicating policy text that already lives under `docs/40-policies/`.

## Contracts:
Domain is the authored SSOT; configs are generated; code consumes generated configs; makefiles call CLI only.

Domain-owned canonical vocabularies are part of SSOT:
- `domain/fastq/artifacts.yaml` and `domain/bam/artifacts.yaml` define allowed artifact IDs.
- `domain/fastq/metrics.yaml` and `domain/bam/metrics.yaml` define allowed metric IDs.
- `bijux-dna domain validate` must fail when stages/tools use IDs outside those vocabularies.

## Examples:
The generated config set is fixed and compiler-owned:
- `configs/ci/registry/tool_registry.toml`
- `configs/ci/stages/stages.toml`
- `configs/ci/tools/images.toml`

Crate authority ownership is defined in:
- `docs/10-architecture/CRATE_AUTHORITY_MAP.md`

## Failure modes:
- Manual edits to generated configs drift from domain and fail CI.
- Makefile-side tool lists drift from registry and fail policies.

## Purpose
This document defines the intended behavior and navigation contract for this topic.

## Scope
Applies only to the files and workflows referenced in this document.

## Non-goals
- Not a replacement for lower-level implementation or crate-specific contracts.

## Contracts
- Content here is normative where explicitly stated.


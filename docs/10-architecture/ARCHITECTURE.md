# Architecture

## Purpose
Define the repository architecture rule for SSOT ownership and consumption boundaries.

## Scope
- Repository-level SSOT ownership across domain data, generated configs, and consumer crates.
- The boundary between authored source, generated artifacts, and runtime/planner behavior.

## Non-goals
- Defining crate-local implementation details.
- Duplicating policy text that already lives under [../40-policies/index.md](../40-policies/index.md).

## Contracts
Domain is the authored SSOT; configs are generated; code consumes generated configs; makes call CLI only.

Domain-owned canonical vocabularies are part of SSOT:
- [../../domain/fastq/artifacts.yaml](../../domain/fastq/artifacts.yaml) and
  [../../domain/bam/artifacts.yaml](../../domain/bam/artifacts.yaml) define allowed artifact IDs.
- [../../domain/fastq/metrics.yaml](../../domain/fastq/metrics.yaml) and
  [../../domain/bam/metrics.yaml](../../domain/bam/metrics.yaml) define allowed metric IDs.
- `bijux-dna domain validate` must fail when stages/tools use IDs outside those vocabularies.

## Examples
The generated config set is fixed and compiler-owned:
- [../../configs/ci/registry/tool_registry.toml](../../configs/ci/registry/tool_registry.toml)
- [../../configs/ci/stages/stages.toml](../../configs/ci/stages/stages.toml)
- [../../configs/ci/tools/images.toml](../../configs/ci/tools/images.toml)

Crate authority ownership is defined in:
- [CRATE_AUTHORITY_MAP.md](CRATE_AUTHORITY_MAP.md)

## Failure modes
- Manual edits to generated configs drift from domain and fail CI.
- Makefile-side tool lists drift from registry and fail policies.

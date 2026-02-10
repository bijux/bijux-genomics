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

## Examples:
The generated config set is fixed and compiler-owned:
- `configs/tool_registry.toml`
- `configs/stages.toml`
- `configs/images.toml`

## Failure modes:
- Manual edits to generated configs drift from domain and fail CI.
- Makefile-side tool lists drift from registry and fail policies.

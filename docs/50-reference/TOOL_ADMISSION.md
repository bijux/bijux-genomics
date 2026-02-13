# TOOL_ADMISSION

Canonical step-by-step process for admitting a tool into production workflows.

## Purpose
Define one authoritative workflow that ties domain -> registry -> containers -> docs -> examples.

## Scope
Applies to all tools entering planned/experimental/production states.

## Non-goals
- Replacing low-level container recipe style docs.
- Replacing domain scientific validation details.

## Contracts
- No tool is production until all steps below are complete and validated.
- Registry/config/container/docs/example surfaces must be mutually consistent.

## Required Path
1. Domain contract: add/update `domain/**/tools/*.yaml` and stage bindings in domain sources.
2. Registry/config contract: regenerate and validate `configs/ci/registry/*`, `configs/ci/tools/*`, and related lock files.
3. Container contract: add/update `containers/**` defs, versions metadata, smoke mappings, and lock outputs.
4. Docs contract: regenerate/update `docs/20-science/TOOL_INDEX.md`, operations notes, and admission references.
5. Example contract: ensure at least one runnable example path is documented/validated where required by policy.
6. Gate contract: run lint/policy checks that enforce parity across all above surfaces.

## Admission Gate
A tool is considered admitted only when registry, containers, QA, and docs are all consistent and CI passes.

## HPC Forward-compat
- If HPC is enabled, container pull/storage roots and output paths may differ from local defaults.
- Admission validity is based on contract parity and pinned artifacts, not local-path assumptions.

## Examples
- Planned -> experimental promotion with registry + container smoke + docs generation update.
- Production promotion only after lock parity and QA matrix pass.

## Failure modes
- Registry entry without container/smoke coverage causes policy failure.
- Docs updated without underlying registry/domain updates causes drift and regeneration failure.

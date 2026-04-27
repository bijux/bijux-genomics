# Contract Authority Ladder

## Purpose
Define a single, explicit authority order for contracts across configs, domain, containers, and docs.

## Scope
Applies to contract decisions for stage/tool/param/runtime/container/docs authority.

## Non-goals
- Replacing schema documents.
- Replacing crate-level API docs.

## Contracts
- If two sources disagree, the higher ladder level wins.
- Lower ladder levels must reference higher ones and cannot silently override them.

## Ladder
1. `domain/**` authored sources (`stages/*.yaml`, `tools/*.yaml`) define domain intent.
2. `configs/ci/**` generated registries define executable CI/runtime contract inputs.
3. `containers/**` definitions and `containers/versions/**` define runtime packaging contract.
4. `docs/**` are normative for process/operations but must not contradict (1)-(3).
5. Generated docs are projections of (1)-(3), never an independent authority.

## Authority Mapping
- Stage/tool/param compatibility: `domain/**` -> generated `configs/ci/**`.
- Required tools and image policy: `configs/ci/tools/*.toml`.
- Build/runtime image realization: `containers/**`.
- Admission and operations workflow: [docs/50-reference/TOOL_ADMISSION.md](../50-reference/TOOL_ADMISSION.md).

## Examples
- If [docs/20-science/TOOL_INDEX.md](../20-science/TOOL_INDEX.md) and
  [configs/ci/registry/tool_registry.toml](../../configs/ci/registry/tool_registry.toml) differ,
  registry wins and docs are regenerated.
- If a container def exists without registry entry, registry policy check fails and def is invalid.

## Failure modes
- Parallel sources of truth create non-deterministic policy outcomes.
- Manual edits to generated docs can mask contract drift.

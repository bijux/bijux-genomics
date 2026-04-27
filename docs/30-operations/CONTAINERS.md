# Containers

## What
Defines container image naming and build rules.

## Why
Container digests make execution reproducible.

## Purpose
Define the operational container contract used by CI, runtime, and tool admission workflows.

## Scope
Applies to registry-linked container definitions, versions, and smoke-policy behavior.

## Non-goals
- Managing registries for the user.

## Contracts
- images.toml must include digests for production runs.
- Docker definitions are `arm64`-only unless policy and checks are updated.
- Container filenames must match generated `containers/TOOL_IDS.txt`.

## HPC Forward-compat
- HPC environments may pull/store containers under site-managed roots.
- Tool/container contract remains registry-driven; only pull/cache location changes.
- Docs and scripts must reference profile-configured paths, not local hardcoded roots.

## Examples
- `bijuxdna/fastp:0.23.4-arm64` with immutable digest.

## Failure modes
- Missing digest blocks promotion to production.
- HPC cache-path assumptions can break container smoke/build workflows.

## Canonical Admission Reference
- Use [docs/50-reference/TOOL_ADMISSION.md](../50-reference/TOOL_ADMISSION.md)
  as the single authoritative "how to add tool" checklist.

## References
- [containers/index.md](../../containers/index.md)
- [containers/docs/index.md](../../containers/docs/index.md)
- [containers/README.md](../../containers/README.md)
- [containers/docs/RELEASE_CHECKLIST.md](../../containers/docs/RELEASE_CHECKLIST.md)
- [containers/docs/PLANNED.md](../../containers/docs/PLANNED.md)

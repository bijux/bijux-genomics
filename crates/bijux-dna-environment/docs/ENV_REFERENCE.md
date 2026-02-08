# ENV_REFERENCE

## Resolution Flow
Input spec → pinned digest → resolved schema object.

## Resolution Precedence (Executable Spec)
1. Explicit digest in spec
2. Versioned image tag
3. Default image tag

## Digest Rules
- If digest is provided, it is authoritative.
- Tag-only references resolve to a digest via the local catalog (no network).

## Fixtures
These examples correspond to `tests/matrix/reference_matrix.rs` fixtures.
- tool_image_spec.json (tag → digest)
- platform_spec.json (platform normalization)

## Deterministic guarantee
Given the same input spec and catalog, resolution is stable and produces the same digest.

## Threat model (stability breakers)
- Changing image catalog contents or digest mappings.
- Changing platform spec defaults (arch, runner, image prefix).
- Modifying resolution precedence or canonicalization.

## Schema evolution
If a spec JSON field changes (name, type, or semantics), bump the schema version and update snapshots.

## Not supported yet
- Network pulls or remote registry probing.
- HPC scheduler integration.

## Example
```json
{
  "tool_id": "fastp",
  "image": "ghcr.io/bijux/fastp:0.23.2",
  "digest": "sha256:deadbeef"
}
```

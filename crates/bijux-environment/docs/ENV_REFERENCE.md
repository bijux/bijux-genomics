# ENV_REFERENCE

## Resolution Precedence (Executable Spec)
1. Explicit digest in spec
2. Versioned image tag
3. Default image tag

## Digest Rules
- If digest is provided, it is authoritative.
- Tag-only references resolve to a digest via the local catalog (no network).

## Fixtures
These examples correspond to `tests/reference_matrix.rs` fixtures.
- tool_image_spec.json (tag → digest)
- platform_spec.json (platform normalization)

## Example
```json
{
  "tool_id": "fastp",
  "image": "ghcr.io/bijux/fastp:0.23.2",
  "digest": "sha256:deadbeef"
}
```

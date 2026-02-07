# ENV_REFERENCE

Resolution precedence:
1. Explicit profile/platform selection.
2. Default platform from environments.toml.
3. Runner kind fallback (docker/apptainer).

Digest rules:
- If a tool entry includes a digest, use it as the image identity.
- If no digest is present, use the resolved tag.

Caching:
- Resolved images are cached per runner under the runtime cache dir.
- Cache key is tool + version/digest + arch.

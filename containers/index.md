# Containers

Purpose: single entrypoint for container contracts, lifecycle, and operations.

## Start Here
- Index: `containers/docs/index.md`
- Tool lifecycle and promotion path: `containers/docs/TOOL_LIFECYCLE.md`
- Version authority: `containers/docs/VERSION_AUTHORITY.md`
- Lock lifecycle: `containers/docs/LOCK_LIFECYCLE.md`
- Smoke contract: `containers/docs/SMOKE_CONTRACT.md`
- Promotion policy: `containers/docs/PROMOTION_POLICY.md`
- Security boundary: `containers/docs/SECURITY_BOUNDARY.md`
- Multiarch policy (arm64-first): `containers/docs/MULTIARCH_POLICY.md`

## Reproducibility
- Source of truth: `containers/versions/versions.toml`
- Lock: `containers/versions/lock.json`
- Versions index hash: `containers/versions/index.sha256`

Hash example:
```text
sha256(containers/versions/versions.toml) = <64-hex>
sha256(containers/versions/lock.json)     = <64-hex>
```

## Contracts
- Runtime/build parity is enforced by container checks under `scripts/containers/`.
- No floating tags for production tools.
- Promotion to production requires locked version metadata and locked digest evidence.

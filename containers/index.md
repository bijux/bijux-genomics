# Containers

Purpose: single entrypoint for container contracts, lifecycle, and operations.

## Start Here
- Index: [containers/docs/index.md](docs/index.md)
- Tool lifecycle and promotion path: [containers/docs/TOOL_LIFECYCLE.md](docs/TOOL_LIFECYCLE.md)
- Version authority: [containers/docs/VERSION_AUTHORITY.md](docs/VERSION_AUTHORITY.md)
- Versioning artifact index: [containers/versions/index.md](versions/index.md)
- Lock lifecycle: [containers/versions/LOCK.md](versions/LOCK.md)
- Smoke contract: [containers/docs/SMOKE_CONTRACT.md](docs/SMOKE_CONTRACT.md)
- Promotion policy: [containers/docs/PROMOTION_POLICY.md](docs/PROMOTION_POLICY.md)
- Science evidence boundary: [containers/docs/SCIENCE_EVIDENCE_BOUNDARY.md](docs/SCIENCE_EVIDENCE_BOUNDARY.md)
- Security boundary: [containers/docs/SECURITY_BOUNDARY.md](docs/SECURITY_BOUNDARY.md)
- Multiarch policy (arm64-first): [containers/docs/MULTIARCH_POLICY.md](docs/MULTIARCH_POLICY.md)
- License assertion inventory: [containers/licenses/README.md](licenses/README.md)

## Reproducibility
- Source of truth: [containers/versions/versions.toml](versions/versions.toml)
- Lock: [containers/versions/lock.json](versions/lock.json)
- Versions index hash: [containers/versions/index.sha256](versions/index.sha256)

Hash example:
```text
sha256(containers/versions/versions.toml) = <64-hex>
sha256(containers/versions/lock.json)     = <64-hex>
```

## Contracts
- Runtime/build parity is enforced by container checks under `bijux-dna-dev/containers/`.
- No floating tags for production tools.
- Promotion to production requires locked version metadata and locked digest evidence.

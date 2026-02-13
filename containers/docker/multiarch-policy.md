# Docker Multiarch Policy

Purpose: define supported Docker architecture contract for this repository.

Scope: `containers/docker/**` definitions and checks under `scripts/containers/`.

Contracts:
- Current supported Docker architecture is `arm64` only.
- `containers/docker/amd64/` may exist only as an empty placeholder unless policy is changed.
- Any multiarch expansion requires:
  - updates to this policy,
  - updates to `scripts/containers/check-docker-arch-policy.sh`,
  - corresponding CI smoke coverage additions.

Cross-build story (amd64 plan while arm64-only):
- Build strategy: use `docker buildx` with explicit platform matrix.
- Publish strategy: publish architecture-qualified tags first, then optional manifest list tags.
- Naming convention: `${tool}:${version}-arm64` and `${tool}:${version}-amd64` before any multiarch alias.
- Promotion criteria: amd64 moves from experimental to production only after smoke parity with arm64.
- Until enabled, checks enforce arm64-only publication contract and reject non-waived amd64 Dockerfiles.

Non-goals:
- Declaring runtime support for host environments outside CI/container contracts.

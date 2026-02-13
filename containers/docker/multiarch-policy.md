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
 - Future amd64 enablement plan must define:
   - build strategy (`buildx` or separate builders),
   - publish strategy (tag naming and manifest list policy),
   - promotion criteria from experimental to production.
 - Until enabled, checks enforce arm64-only publication contract.

Non-goals:
- Declaring runtime support for host environments outside CI/container contracts.

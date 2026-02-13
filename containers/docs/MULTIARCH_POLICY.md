# Multiarch Policy

Purpose: define architecture support contract.

## Current Contract
- Docker support is `arm64`-first and authoritative today.
- `amd64` is optional and must be explicitly enabled by policy/config.

## Practical Rule
- A tool is considered docker-supported if `containers/docker/arm64/Dockerfile.<tool>` exists.
- Any `amd64` enablement must preserve naming parity and policy checks.

## References
- `containers/docker/multiarch-policy.md`
- `scripts/containers/check-docker-arch-policy.sh`
- `scripts/containers/check-docker-arm64-completeness.sh`

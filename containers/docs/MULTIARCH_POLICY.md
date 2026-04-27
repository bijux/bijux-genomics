# Multiarch Policy

Purpose: define architecture support contract.

Authority surfaces:
- [../README.md](../README.md)
- [../index.md](../index.md)
- [../docker/multiarch-policy.md](../docker/multiarch-policy.md)

## Current Contract
- Docker support is `arm64`-first and authoritative today.
- `amd64` is optional and must be explicitly enabled by policy/config.

## Practical Rule
- A tool is considered docker-supported if `containers/docker/arm64/Dockerfile.<tool>` exists.
- Any `amd64` enablement must preserve naming parity and policy checks.

## References
- docker multiarch policy file above
- `cargo run -p bijux-dna-dev -- containers run check-docker-arch-policy`
- `cargo run -p bijux-dna-dev -- containers run check-docker-arm64-completeness`

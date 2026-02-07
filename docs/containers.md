# Containers

One tool = one Dockerfile = one image.

Naming convention (current):
- `containers/docker/<arch>/<tool>.Dockerfile`
  - Example: `containers/docker/arm64/fastp.Dockerfile`

Example image name:
- `bijuxdna/fastp:0.23.4-arm64`

Build images (Docker):
- `cargo run -p bijux-environment --bin build_docker_images -- --platform docker-mac-arm64`

Test images (Docker):
- `cargo run -p bijux-environment --bin test_docker_images -- --platform docker-mac-arm64`

Digest policy
- `configs/images.toml` should include immutable `sha256:` digests for all published images.
- For local development, version tags are accepted; production runs must use digests.
- When a digest is missing, treat the image as non‑reproducible and block promotion.

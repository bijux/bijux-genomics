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

TODO (digests)
- We can’t populate `sha256:` repo digests for all images without pushing them to a registry.
- Missing digests currently: `bwa`, `seqtk`, `fastqvalidator`, `fqtools`.
- Once registry access is available, build + push, then update `configs/images.yaml` with repo digests.

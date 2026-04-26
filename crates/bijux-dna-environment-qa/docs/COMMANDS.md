# bijux-dna-environment-qa Commands

`bijux-dna-environment-qa` is the effectful environment QA crate. It owns QA command surfaces and
host command orchestration that must not leak into production crates.

## Managed Command Inventory

- `image_qa`: Cargo binary at `src/bin/image_qa.rs`; runs image QA for the selected platform through
  `image_qa::run_image_qa`.
- `qa_docker_images`: Cargo binary at `src/bin/qa_docker_images.rs`; probes Docker images from the
  tool catalog and reports pass/fail status.
- `build_docker_images`: Cargo binary at `src/bin/build_docker_images.rs`; builds Docker images from
  configured Dockerfiles and runs version smoke checks.

## Host Commands Managed By This Crate

- `docker build`: builds catalog images in `build_docker_images`.
- `docker run`: runs version probes, image QA scenarios, seqkit probes, and container smoke checks.
- `docker image inspect`: checks Docker image presence.
- `docker images -q`: skips existing images in the build command.
- `docker rm`, `docker wait`, `docker logs`, `docker inspect`: lifecycle helpers for QA containers.
- `gzip -t`: verifies gzip integrity for FASTQ outputs.
- `git rev-parse HEAD`: captures OCI revision metadata for image builds.
- `date -u +%Y-%m-%dT%H:%M:%SZ`: captures OCI creation metadata for image builds.
- Apptainer smoke QA delegates to `bijux-dna-environment::api::run_smoke_script_batch`, which
  invokes the developer-control-plane smoke command.

## Offline Defaults

Fast tests must not run Docker, Apptainer, network pulls, or long-running QA commands. Runtime QA is
explicit operator work and should write under `artifacts/image-qa/<platform>/`.

## Process Ownership Files

- `src/bin/build_docker_images.rs`
- `src/image_qa/datasets/hydration.rs`
- `src/image_qa/qa_docker_images/runtime.rs`
- `src/image_qa/support/docker_exec/inspection.rs`
- `src/image_qa/support/docker_exec/merge.rs`
- `src/image_qa/support/docker_exec/transform.rs`
- `src/image_qa/support/docker_runtime.rs`
- `src/image_qa/support/seqkit.rs`

## Verification

Use:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment-qa --no-default-features --test boundaries
```

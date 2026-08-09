# bijux-dna-containers-qa Commands

`bijux-dna-containers-qa` is the effectful environment QA crate. It owns QA command surfaces and
host command orchestration that must not leak into production crates.

## Managed Command Inventory

- `image_qa`: Cargo binary at `src/bin/image_qa.rs`; runs image QA for the selected platform through
  `image_qa::run_image_qa`.
- `qa_docker_images`: Cargo binary at `src/bin/qa_docker_images.rs`; probes Docker images from the
  tool catalog and reports pass/fail status.
- `build_docker_images`: Cargo binary at `src/bin/build_docker_images.rs`; builds Docker images from
  configured Dockerfiles and runs version smoke checks.
- `runtime_verification`: Cargo binary at `src/bin/runtime_verification.rs`; executes bounded
  version/help probes for the exact catalog-locked Apptainer inventory or verifies its report. The
  optional `--baseline <publication>` input reuses unchanged passing tool evidence only after
  checking both referenced identity and manifest files against their recorded SHA-256 digests.
- `security_verification`: Cargo binary at `src/bin/security_verification.rs`; collects complete
  raw security evidence for exact catalog-locked Apptainer images without deriving policy passes.
  The optional `--reuse-publication <publication>` input admits every referenced baseline artifact,
  reuses records with unchanged image and tool-contract identities, and fully probes changed tools.

## Host Commands Managed By This Crate

- `docker build`: builds catalog images in `build_docker_images`.
- `docker run`: runs version probes, image QA scenarios, seqkit probes, and container smoke checks.
- `docker image inspect`: checks Docker image presence.
- `docker images -q`: skips existing images in the build command.
- `docker rm`, `docker wait`, `docker logs`, `docker inspect`: lifecycle helpers for QA containers.
- `gzip -t`: verifies gzip integrity for FASTQ outputs.
- `git rev-parse HEAD`: captures OCI revision metadata for image builds.
- `date -u +%Y-%m-%dT%H:%M:%SZ`: captures OCI creation metadata for image builds.
- Apptainer catalog QA composes `bijux-dna-containers::api::verify_apptainer_tool_contract` and
  `run_apptainer_smoke`; this crate owns catalog iteration, concurrency, and publication.
- `syft scan`: emits a complete CycloneDX JSON SBOM from the exact SIF.
- `grype sbom`: scans that SBOM against the admitted immutable Grype database.
- `gitleaks detect`: scans a bounded sandbox extraction with redaction and full JSON output.
- `apptainer verify`, `apptainer build --sandbox`, and `apptainer exec`: collect signature,
  filesystem privilege, non-root, read-only-system, and isolated-network observations.

## Offline Defaults

Fast tests must not run Docker, Apptainer, network pulls, or long-running QA commands. Image QA
writes under `artifacts/image-qa/<platform>/`; catalog runtime verification writes only to its
explicit operator-provided study evidence root.

## Process Ownership Files

- `src/bin/build_docker_images.rs`
- `src/image_qa/datasets/hydration.rs`
- `src/image_qa/qa_docker_images/runtime.rs`
- `src/image_qa/support/docker_exec/inspection.rs`
- `src/image_qa/support/docker_exec/merge.rs`
- `src/image_qa/support/docker_exec/transform.rs`
- `src/image_qa/support/docker_runtime.rs`
- `src/image_qa/support/seqkit.rs`
- `src/security_verification/workflow.rs`

## Verification

Use:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers-qa --no-default-features --test boundaries
```

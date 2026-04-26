# bijux-dna-environment-qa Image QA

This document is the image-QA operator map. It consolidates the runbook, matrix, dataset, Apptainer,
and artifact rules for this crate.

## Runbook

Use artifact-rooted commands:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -p bijux-dna-environment-qa --bin qa_docker_images -- --platform docker-amd64
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -p bijux-dna-environment-qa --bin image_qa -- --platform docker-amd64
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -p bijux-dna-environment-qa --bin build_docker_images -- --platform docker-amd64 --skip-existing
```

## QA Matrix

- Static QA: image presence and digest/tag resolution.
- Docker image probe QA: executable and version command checks for catalog images.
- Behavioral QA: FASTQ trim, validate, filter, merge, correct, report, UMI, and stats scenarios.
- Apptainer QA: registry-driven smoke contract through the environment smoke helper.

## Pass Criteria

- Image exists for the resolved platform.
- Required executable or probe command succeeds with an accepted exit code.
- Probe output contains the expected version when a version probe is declared.
- Behavioral outputs satisfy execution contracts and deterministic sanity checks.
- QA records are persisted for the stage/tool/platform/input tuple.

## Dataset Policy

- Fast tests use local fixtures only.
- Runtime QA may use operator-provided external datasets, but default tests never fetch them.
- Synthetic fixture artifacts live under `tests/fixtures/qa_artifacts/default/`.
- Larger real-world datasets must record source, checksum, license, and purpose outside source code
  before use.

## Apptainer Scope

Apptainer QA is not a placeholder. It is delegated through
`bijux-dna-environment::api::run_smoke_script_batch` and uses the runtime manifest tool roster.
Parity means Apptainer smoke output must satisfy the same stage/tool readiness intent as Docker
image probes, while preserving Apptainer-specific image/cache behavior.

## Artifact Contract

Runtime QA records write under `artifacts/image-qa/<platform>/`; per-run tool outputs write under
`artifacts/image-qa/runs/<stage>/`. Fixture artifacts use production-like names (`manifest.json`,
`report.json`) and are checked by
`tests/contracts/artifacts/qa_artifact_contract.rs`.

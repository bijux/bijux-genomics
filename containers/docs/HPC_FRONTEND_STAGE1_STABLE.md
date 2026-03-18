# HPC Frontend Stage 1 Stable

Recorded at: `2026-02-13T21:02:31Z`  
Repository head at freeze: `2303f4f7`

## Scope
This freeze marks governance checks for container policy Stage 1 on HPC frontend workflows.

## Evidence
- `cargo run -p bijux-dev-dna -- containers run check-tool-id-manifest` -> `OK`
- `scripts/containers/check-tool-id-contract.sh` -> `OK`
- `scripts/checks/check-domain-tool-parity.sh` -> `OK`
- `scripts/containers/check-tool-container-coverage.sh` -> `OK`
- `scripts/containers/check-non-bijux-sources.sh` -> `OK`
- `cargo run -p bijux-dev-dna -- containers run check-promotion-policy` -> `OK`
- `scripts/checks/check-deprecations-enforcement.sh` -> `OK`
- `cargo run -p bijux-dev-dna -- containers run check-version-deprecations` -> `OK`
- `cargo run -p bijux-dev-dna -- containers run check-version-lock` -> `OK`
- `scripts/containers/check-docker-labels.sh` -> `OK`
- `scripts/containers/check-registry-vs-defs.sh` -> `OK`
- `scripts/checks/check-container-ssot-parity.sh` -> `OK`
- `scripts/checks/check-registry-required-tools-parity.sh` -> `OK`
- `cargo run -p bijux-dev-dna -- containers run check-qa-matrix-generated` -> `OK`
- `scripts/containers/check-dockerfiles-built.sh` -> `SKIP (CI-only gate)`

## QA Matrix
- Generated/validated by:
  - `cargo run -p bijux-dev-dna -- containers run generate-qa-matrix`
  - `cargo run -p bijux-dev-dna -- containers run check-qa-matrix-generated`
- Reference:
  - `docs/30-operations/APPTAINER_QA_MATRIX.md`

## Stable Marking Rule
Stage 1 remains stable only while all non-CI-gated checks above remain passing and release-gate policies continue to enforce promotion/deprecation/version-lock contracts.

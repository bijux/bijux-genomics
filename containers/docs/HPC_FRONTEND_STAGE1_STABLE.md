# HPC Frontend Stage 1 Stable

Recorded at: `2026-02-13T21:02:31Z`  
Repository head at freeze: `2303f4f7`

## Scope
This freeze marks governance checks for container policy Stage 1 on HPC frontend workflows.

## Evidence
- `scripts/containers/check-tool-id-manifest.sh` -> `OK`
- `scripts/containers/check-tool-id-contract.sh` -> `OK`
- `scripts/checks/check-domain-tool-parity.sh` -> `OK`
- `scripts/containers/check-tool-container-coverage.sh` -> `OK`
- `scripts/containers/check-non-bijux-sources.sh` -> `OK`
- `scripts/containers/check-promotion-policy.sh` -> `OK`
- `scripts/checks/check-deprecations-enforcement.sh` -> `OK`
- `scripts/containers/check-version-deprecations.sh` -> `OK`
- `scripts/containers/check-version-lock.sh` -> `OK`
- `scripts/containers/check-docker-labels.sh` -> `OK`
- `scripts/containers/check-registry-vs-defs.sh` -> `OK`
- `scripts/checks/check-container-ssot-parity.sh` -> `OK`
- `scripts/checks/check-registry-required-tools-parity.sh` -> `OK`
- `scripts/containers/check-qa-matrix-generated.sh` -> `OK`
- `scripts/containers/check-dockerfiles-built.sh` -> `SKIP (CI-only gate)`

## QA Matrix
- Generated/validated by:
  - `scripts/containers/generate-qa-matrix.sh`
  - `scripts/containers/check-qa-matrix-generated.sh`
- Reference:
  - `docs/30-operations/APPTAINER_QA_MATRIX.md`

## Stable Marking Rule
Stage 1 remains stable only while all non-CI-gated checks above remain passing and release-gate policies continue to enforce promotion/deprecation/version-lock contracts.

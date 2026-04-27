# HPC Frontend Stage 1 Stable

Recorded at: `2026-02-13T21:02:31Z`  
Repository head at freeze: `2303f4f7`

## Scope
This freeze marks governance checks for container policy Stage 1 on HPC frontend workflows.

Authority surfaces:
- [../README.md](../README.md)
- [FRONTEND_BUILD_AUTHORITY.md](FRONTEND_BUILD_AUTHORITY.md)
- [TOOL_IDS_CONTRACT.md](TOOL_IDS_CONTRACT.md)
- [VERSION_AUTHORITY.md](VERSION_AUTHORITY.md)
- [../versions/LOCK.md](../versions/LOCK.md)
- [../../docs/30-operations/APPTAINER_QA_MATRIX.md](../../docs/30-operations/APPTAINER_QA_MATRIX.md)

## Evidence
- `cargo run -p bijux-dna-dev -- containers run check-tool-id-manifest` -> `OK`
- `cargo run -p bijux-dna-dev -- containers run check-tool-id-contract` -> `OK`
- `cargo run -q -p bijux-dna-dev -- checks run check-domain-tool-parity` -> `OK`
- `cargo run -p bijux-dna-dev -- containers run check-tool-container-coverage` -> `OK`
- `cargo run -p bijux-dna-dev -- containers run check-non-bijux-sources` -> `OK`
- `cargo run -p bijux-dna-dev -- containers run check-promotion-policy` -> `OK`
- `cargo run -q -p bijux-dna-dev -- checks run check-deprecations-enforcement` -> `OK`
- `cargo run -p bijux-dna-dev -- containers run check-version-deprecations` -> `OK`
- `cargo run -p bijux-dna-dev -- containers run check-version-lock` -> `OK`
- `cargo run -p bijux-dna-dev -- containers run check-docker-labels` -> `OK`
- `cargo run -p bijux-dna-dev -- containers run check-registry-vs-defs` -> `OK`
- `cargo run -q -p bijux-dna-dev -- checks run check-container-ssot-parity` -> `OK`
- `cargo run -q -p bijux-dna-dev -- checks run check-registry-required-tools-parity` -> `OK`
- `cargo run -p bijux-dna-dev -- containers run check-qa-matrix-generated` -> `OK`
- `cargo run -p bijux-dna-dev -- containers run check-dockerfiles-built` -> `SKIP (CI-only gate)`

## QA Matrix
- Generated/validated by:
  - `cargo run -p bijux-dna-dev -- containers run generate-qa-matrix`
  - `cargo run -p bijux-dna-dev -- containers run check-qa-matrix-generated`
- Reference:
  - `docs/30-operations/APPTAINER_QA_MATRIX.md`

## Stable Marking Rule
Stage 1 remains stable only while all non-CI-gated checks above remain passing and release-gate policies continue to enforce promotion/deprecation/version-lock contracts.

# Frontend Build Authority

Purpose: enforce HPC frontend nodes as the only authority for Apptainer SIF builds.

[../README.md](../README.md), [../versions/LOCK.md](../versions/LOCK.md), and
[../../docs/30-operations/TRACEABILITY_PROOF_FRONTEND.md](../../docs/30-operations/TRACEABILITY_PROOF_FRONTEND.md)
define the adjacent control and proof surfaces for this frontend-only build
authority.

## Doctrine
- Build all Apptainer SIF images on HPC frontend/login nodes only.
- Never run Apptainer `%post` build steps on compute nodes.
- Frontend builds must pass pinned-version checks from
  [containers/versions/versions.toml](../versions/versions.toml).
- Frontend-built SIF SHA256 values are authoritative and recorded in `artifacts/containers/hpc/frontend-sif-digests.json`.
- Lock metadata carries frontend digest authority in
  [containers/versions/LOCK.md](../versions/LOCK.md) under
  `items[].frontend_resolved_sif_sha256`.

## Enforcement
- Runtime guard scripts:
  - `cargo run -p bijux-dna-dev -- containers run build-apptainer-hpc-frontend`
  - `cargo run -p bijux-dna-dev -- containers run build-apptainer-all`
  - `cargo run -p bijux-dna-dev -- containers run smoke-apptainer`
- CI policy checks:
  - `cargo run -p bijux-dna-dev -- containers run check-apptainer-post-pins`
  - `cargo run -p bijux-dna-dev -- containers run check-hpc-frontend-policy-enforcement`
- Cache policy checks:
  - `cargo run -p bijux-dna-dev -- containers run check-apptainer-cache-policy`
  - [configs/ci/tools/apptainer_cache_policy.toml](../../configs/ci/tools/apptainer_cache_policy.toml)

## Comparison Workflow
1. Build on frontend with `cargo run -p bijux-dna-dev -- containers run build-apptainer-hpc-frontend`.
2. Generate local digests with `cargo run -p bijux-dna-dev -- containers run generate-local-apptainer-digests`.
3. Compare with `cargo run -p bijux-dna-dev -- containers run compare-frontend-local-sif-hash`.
4. If mismatch exists, capture deterministic cause (base digest drift, embedded timestamp, host/runtime variation, or source artifact change).

## Full Frontend Smoke Workflow
1. Run `cargo run -p bijux-dna-dev -- containers run run-apptainer-frontend-smoke`.
2. Smoke executes `--version` and `--help` plus contract probes for every Apptainer tool SIF.
3. Logs and manifests are stored under `artifacts/containers/hpc/frontend-smoke/`.
4. Proof gate is enforced by:
   - `cargo run -p bijux-dna-dev -- containers run check-apptainer-frontend-smoke-proof`
   - `cargo run -p bijux-dna-dev -- containers run check-apptainer-frontend-version-output-lock`

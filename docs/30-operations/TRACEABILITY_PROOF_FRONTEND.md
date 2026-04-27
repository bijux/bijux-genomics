# Frontend Traceability Proof

## Purpose
Given `report.html` (or `report.json`) from a frontend mini run, trace back to exact tool/container/config authority.

Use [../../containers/docs/FRONTEND_BUILD_AUTHORITY.md](../../containers/docs/FRONTEND_BUILD_AUTHORITY.md)
for the HPC-only build doctrine and
[../../containers/versions/LOCK.md](../../containers/versions/LOCK.md) for the
lock semantics that this proof verifies.

## Scope
Defines required inputs and deterministic proof steps for frontend mini runs.

## Non-goals
- Defining report rendering or dashboard UX behavior.

## Contracts
- Proof must bind run artifacts to lock hash, domain hash, config hash, and tool digests.
- Missing traceability fields are a contract violation.

## Inputs
- `artifacts/hpc/frontend-mini-e2e/<run-id>/<mini>/report.json`
- `artifacts/hpc/frontend-mini-e2e/<run-id>/<mini>/frontend_run_meta.json`
- [containers/versions/lock.json](../../containers/versions/lock.json)
- `artifacts/containers/hpc/frontend-smoke/summary.json`

## Procedure
1. Read `frontend_run_meta.json`.
2. Capture:
   - `container_lock_sha256`
   - `domain_hash_sha256`
   - `config_hash_sha256`
   - `tool_versions_ref`
3. Verify `container_lock_sha256` equals SHA256 of
   [containers/versions/lock.json](../../containers/versions/lock.json).
4. Open `tool_versions_ref` and map tool IDs to:
   - `resolved_image_digest` / `sif_digest_sha256`
   - version outputs.
5. If VCF downstream used panels, verify panel lock artifacts from:
   - [configs/vcf/panels/panels.toml](../../configs/vcf/panels/panels.toml)
   - [configs/vcf/panels/locks/lock.json](../../configs/vcf/panels/locks/lock.json)
6. Confirm timestamps and status:
   - `start_utc`, `end_utc`, `exit_code`.

## Expected Outcome
Every mini run can be traced from report artifact to:
- tool digest,
- lock authority,
- domain snapshot hash,
- config hash,
- runtime version evidence.

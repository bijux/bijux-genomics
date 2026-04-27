# Slurm Phase Entry Criteria

## Purpose
Defines the mandatory entry gate for enabling Slurm execution phase.

## Scope
Specifies required green checks and evidence artifacts before starting Slurm work.

## Non-goals
- Defining Slurm job templates or scheduler tuning parameters.

## Contracts
- Slurm phase must not start until every required green-state check passes.
- Evidence artifacts listed here must exist and be current.

[HPC_FRONTEND_RUNBOOK.md](HPC_FRONTEND_RUNBOOK.md),
[TRACEABILITY_PROOF_FRONTEND.md](TRACEABILITY_PROOF_FRONTEND.md), and
[../../containers/docs/FRONTEND_BUILD_AUTHORITY.md](../../containers/docs/FRONTEND_BUILD_AUTHORITY.md)
define the frontend proof surfaces that must be closed before Slurm admission.

## Required Green State
All items below must be true before enabling Slurm execution phase:

1. Frontend all-tools container workflow is green:
   - `cargo run -p bijux-dna-dev -- containers run apptainer-build-all`
   - smoke contracts, lock checks, SBOM/license/provenance checks pass.
2. Two frontend mini E2E runs are green:
   - VCF downstream mini (`vcf_downstream_vcf_full_mini`)
   - eDNA mini (`fastq_edna_mini`)
3. Observability proof checks are green:
   - artifact/report contracts
   - telemetry sanity
   - traceability proof fields present.
4. Lock and promotion state is current:
   - [containers/versions/LOCK.md](../../containers/versions/LOCK.md) refreshed
   - production tools promoted only through lifecycle scripts.

## Evidence Artifacts
- `artifacts/containers/hpc/frontend-smoke/summary.json`
- `artifacts/hpc/frontend-mini-e2e/<run-id>/summary.json`
- [containers/docs/APPTAINER_FRONTEND_SECURITY_SUMMARY.md](../../containers/docs/APPTAINER_FRONTEND_SECURITY_SUMMARY.md)
- [containers/docs/APPTAINER_FRONTEND_REPRODUCIBILITY_REPORT.md](../../containers/docs/APPTAINER_FRONTEND_REPRODUCIBILITY_REPORT.md)

## Notes
This document defines only the gate to start Slurm phase.

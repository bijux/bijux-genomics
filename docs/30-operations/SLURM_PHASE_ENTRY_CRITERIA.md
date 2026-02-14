# Slurm Phase Entry Criteria

## Required Green State
All items below must be true before enabling Slurm execution phase:

1. Frontend all-tools container workflow is green:
   - `scripts/containers/apptainer-build-all.sh`
   - smoke contracts, lock checks, SBOM/license/provenance checks pass.
2. Two frontend mini E2E runs are green:
   - VCF downstream mini (`vcf_downstream_vcf_full_mini`)
   - eDNA mini (`fastq_edna_mini`)
3. Observability proof checks are green:
   - artifact/report contracts
   - telemetry sanity
   - traceability proof fields present.
4. Lock and promotion state is current:
   - `containers/versions/lock.json` refreshed
   - production tools promoted only through lifecycle scripts.

## Evidence Artifacts
- `artifacts/containers/hpc/frontend-smoke/summary.json`
- `artifacts/hpc/frontend-mini-e2e/<run-id>/summary.json`
- `containers/docs/APPTAINER_FRONTEND_SECURITY_SUMMARY.md`
- `containers/docs/APPTAINER_FRONTEND_REPRODUCIBILITY_REPORT.md`

## Non-goal
This document does not define Slurm job templates; it defines the gate to start that phase.

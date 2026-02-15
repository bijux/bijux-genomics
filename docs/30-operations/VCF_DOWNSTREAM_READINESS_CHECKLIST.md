# VCF Downstream Readiness Checklist

## Purpose
Defines required gates for promoting VCF downstream workflows to compute-node execution readiness.

## Scope
Gate for promoting VCF downstream runs from frontend-only validation to compute-node execution.

## Non-goals
- Defining per-stage implementation details already covered by stage contracts.

## Contracts
- Every required check in this file must pass for readiness.
- Production badge eligibility requires lock validity and acceptance criteria success.

## Required Green Checks
- `scripts/checks/check-vcf-reference-governance.sh`
- `scripts/checks/check-vcf-compatibility-matrix.sh`
- `scripts/checks/check-vcf-deprecation-lifecycle.sh`
- `scripts/checks/check-registry-required-tools-parity.sh`
- `scripts/checks/check-param-registry-completeness.sh`
- `scripts/checks/check-frontend-mini-artifacts.sh`
- `scripts/checks/check-frontend-observability-proof.sh`
- `scripts/checks/check-frontend-telemetry-sanity.sh`

## Stage Acceptance Criteria Source
- `configs/vcf/downstream_acceptance.toml`

## Production Badge Rule
A run is production-badge eligible only when all are true:
- panel/map locks are resolved and validated
- `decision.imputation_accept` is accepted
- stage acceptance criteria in `configs/vcf/downstream_acceptance.toml` are met

## Fail-Fast Refusals
- unknown phasing/impute knobs
- contig/ploidy/field coercions
- unresolved species context or reference bundle
- unresolved panel/map locks

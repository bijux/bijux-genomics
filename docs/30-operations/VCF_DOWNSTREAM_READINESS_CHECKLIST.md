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
- `./scripts/run.sh checks check-vcf-reference-governance`
- `./scripts/run.sh checks check-vcf-compatibility-matrix`
- `./scripts/run.sh checks check-vcf-deprecation-lifecycle`
- `./scripts/run.sh checks check-registry-required-tools-parity`
- `./scripts/run.sh checks check-param-registry-completeness`
- `./scripts/run.sh checks check-frontend-mini-artifacts`
- `./scripts/run.sh checks check-frontend-observability-proof`
- `./scripts/run.sh checks check-frontend-telemetry-sanity`

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

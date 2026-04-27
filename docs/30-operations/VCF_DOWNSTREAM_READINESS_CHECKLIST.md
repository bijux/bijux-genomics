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
- Lock validity is governed by [VCF_REFERENCE_CACHE_POLICY.md](VCF_REFERENCE_CACHE_POLICY.md).
- Frontend proof remains governed by [TRACEABILITY_PROOF_FRONTEND.md](TRACEABILITY_PROOF_FRONTEND.md).

## Required Green Checks
- `cargo run -q -p bijux-dna-dev -- checks run check-vcf-reference-governance`
- `cargo run -q -p bijux-dna-dev -- checks run check-vcf-compatibility-matrix`
- `cargo run -q -p bijux-dna-dev -- checks run check-vcf-deprecation-lifecycle`
- `cargo run -q -p bijux-dna-dev -- checks run check-registry-required-tools-parity`
- `cargo run -q -p bijux-dna-dev -- checks run check-param-registry-completeness`
- `cargo run -q -p bijux-dna-dev -- checks run check-frontend-mini-artifacts`
- `cargo run -q -p bijux-dna-dev -- checks run check-frontend-observability-proof`
- `cargo run -q -p bijux-dna-dev -- checks run check-frontend-telemetry-sanity`

## Stage Acceptance Criteria Source
- [configs/vcf/downstream_acceptance.toml](../../configs/vcf/downstream_acceptance.toml)

## Production Badge Rule
A run is production-badge eligible only when all are true:
- panel/map locks are resolved and validated
- `decision.imputation_accept` is accepted
- stage acceptance criteria in
  [configs/vcf/downstream_acceptance.toml](../../configs/vcf/downstream_acceptance.toml) are met

## Fail-Fast Refusals
- unknown phasing/impute knobs
- contig/ploidy/field coercions
- unresolved species context or reference bundle
- unresolved panel/map locks

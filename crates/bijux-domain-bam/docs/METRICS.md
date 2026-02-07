# METRICS

Required fields:
- units
- thresholds
- evidence counts

Insufficient evidence:
- missing CI or sample count below threshold.

## Checklist: add a new BAM metric
- Define the metric schema and semantics in `src/metrics/*`.
- Update invariant rules in `src/invariants/*`.
- Refresh completeness tests and stage contract snapshots under `tests/contracts/*`.

# METRICS

Required fields:
- units
- thresholds
- evidence counts

Insufficient evidence:
- missing CI or sample count below threshold.

## Glossary
See `docs/METRICS_GLOSSARY.md` for definitions and links to code modules.

## Checklist: add a new BAM metric
- Define the metric schema and semantics in `src/metrics/*`.
- Update invariant rules in `src/invariants/*`.
- Refresh completeness tests and stage contract snapshots under `tests/contracts/*`.

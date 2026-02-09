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

## Damage/authentication typed outputs
Damage/auth metrics are modeled with a shared core plus tool-specific envelopes:
- `DamageCoreFieldsV1` (shared core subset)
- `DamageProfilerMetricsV1`
- `PmdtoolsMetricsV1`
- `NgsBriggsMetricsV1`
- `AdDeamMetricsV1`

Shared concepts captured as typed schemas:
- misincorporation summaries (`MisincorporationCurveSummaryV1`, C→T and G→A by position)
- PMD score distributions (`PmdScoreDistributionV1`)
- inferred tool parameters where available (`lambda`, `delta_s`, clustering count)

## Contamination typed outputs
Contamination metrics now model operator-required context:
- `ContaminationToolMetricsV1`
- per-tool wrappers: `SchmutziMetricsV1`, `VerifyBamId2MetricsV1`, `ContamMixMetricsV1`

Required fields include:
- estimate + confidence interval (`estimate`, `ci_low`, `ci_high`)
- model assumptions
- required input context (`reference_panel`, mt/nuclear scope)
- warnings/quality flags

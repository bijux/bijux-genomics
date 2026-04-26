# bijux-dna-domain-bam Metrics

BAM metrics must define units, thresholds, evidence counts, and insufficient-evidence behavior. Missing confidence intervals, missing supporting counts, or sample counts below threshold should produce explicit insufficient-evidence states rather than silent pass/fail shortcuts.

## Metric map

- alignment: mapping counts and duplication summaries. Module: `src/metrics/pre/alignment.rs`.
- mapq: mapping quality distribution. Module: `src/metrics/pre/mapq.rs`.
- idxstats: per-contig mapping summary. Module: `src/metrics/pre/idxstats.rs`.
- fragment_length: length distribution and summary stats. Module: `src/metrics/pre/fragment.rs`.
- coverage and coverage_uniformity: breadth, mean, median, coefficient of variation, and dropout fraction. Module: `src/metrics/core/coverage.rs`.
- damage: C-to-T/G-to-A patterns and PMD summaries. Module: `src/metrics/core/damage.rs`.
- authenticity: evidence, score, confidence, and rationale. Module: `src/metrics/downstream/authenticity.rs`.
- complexity: saturation estimates. Module: `src/metrics/core/complexity.rs`.
- contamination: estimate, confidence interval, model, scope, and warnings. Module: `src/metrics/downstream/contamination.rs`.
- sex: classification and confidence. Module: `src/metrics/downstream/sex.rs`.
- genotyping: call rate and posterior summaries. Module: `src/metrics/downstream/genotyping.rs`.
- sufficiency verdicts: coverage, kinship, and sex sufficiency flags. Module: `src/metrics/downstream/sufficiency.rs`.

## Interpretation

- Authenticity requires expected damage patterns and fragment characteristics; insufficient evidence must remain distinct from failure.
- Contamination interpretation uses estimates, confidence intervals, scope, assumptions, and warning flags.
- Sex inference requires enough data and stable confidence bounds before classification.
- Kinship requires sufficient coverage and marker overlap before reporting.

## Tool families

Damage and authenticity typed outputs share a core plus tool-specific envelopes:
- `DamageCoreFieldsV1`
- `DamageProfilerMetricsV1`
- `PmdtoolsMetricsV1`
- `NgsBriggsMetricsV1`
- `AdDeamMetricsV1`

Contamination typed outputs model operator-required context:
- `ContaminationToolMetricsV1`
- `SchmutziMetricsV1`
- `VerifyBamId2MetricsV1`
- `ContamMixMetricsV1`

Reference tool families include mapDamage2 for damage profiling, pydamage for authenticity, and established contamination-estimation methods.

## Checklist: add a new BAM metric
- Define the metric schema and semantics in `src/metrics/*`.
- Update invariant rules in `src/invariants/*`.
- Refresh completeness tests and stage contract snapshots under `tests/contracts/*`.

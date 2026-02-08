# METRICS_GLOSSARY

This glossary defines the core BAM metrics and points to the owning code modules.

## Alignment + mapping
- alignment: mapping counts and duplication summaries.
  Module: `src/metrics/pre/alignment.rs`
- mapq: mapping quality distribution.
  Module: `src/metrics/pre/mapq.rs`
- idxstats: per-contig mapping summary.
  Module: `src/metrics/pre/idxstats.rs`

## Fragment + coverage
- fragment_length: length distribution and summary stats.
  Module: `src/metrics/pre/fragment.rs`
- coverage: breadth/mean/median coverage.
  Module: `src/metrics/core/coverage.rs`
- coverage_uniformity: coefficient of variation + dropout fraction.
  Module: `src/metrics/core/coverage.rs`

## Damage + authenticity
- damage: c→t/g→a patterns and PMD summaries.
  Module: `src/metrics/core/damage.rs`
- authenticity: evidence + score + rationale.
  Module: `src/metrics/downstream/authenticity.rs`

## Complexity + contamination
- complexity: saturation estimates.
  Module: `src/metrics/core/complexity.rs`
- contamination: estimate + CI + method.
  Module: `src/metrics/downstream/contamination.rs`

## Downstream sufficiency
- sex: classification + confidence.
  Module: `src/metrics/downstream/sex.rs`
- genotyping: call rate + posterior summaries.
  Module: `src/metrics/downstream/genotyping.rs`
- sufficiency verdicts: coverage/kinship/sex sufficiency flags.
  Module: `src/metrics/downstream/sufficiency.rs`

# INTERPRETATION

Downstream verdicts interpret core metrics. Thresholds are defined in the metric schemas and
invariant rules under `src/metrics/*` and `src/invariants/*`.

## Authenticity
A high authenticity score indicates expected damage patterns and fragment characteristics.
Threshold meaning: score/confidence fields are compared to invariant limits; warnings appear
when evidence is insufficient (see `src/metrics/downstream/authenticity.rs`).

## Contamination
High contamination estimates warrant filtering or reprocessing.
Threshold meaning: confidence intervals and sufficiency flags determine verdicts; large CI
widths trigger "insufficient" rather than "fail" (see `src/metrics/downstream/contamination.rs`).

## Sex inference
Use confidence thresholds before reporting.
Threshold meaning: `sufficient_data` and confidence bounds control classification stability
(see `src/metrics/downstream/sex.rs`).

## Kinship
Requires sufficient coverage and marker overlap.
Threshold meaning: overlap and coverage sufficiency flags determine whether kinship is reported
(see `src/metrics/downstream/sufficiency.rs`).

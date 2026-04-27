# FASTQ Scientific Spec

## What
Describes scientific expectations for FASTQ pipelines.

## Why
Ensures reproducible and interpretable preprocessing.

## Non-goals
- Tool‑specific tuning guidance.

## Contracts
- Stage purpose and interpretation live in
  [METHODOLOGICAL_INTENT.md](METHODOLOGICAL_INTENT.md).
- Stage-by-stage inputs, outputs, and defaults live in
  [STAGE_CATALOG.md](STAGE_CATALOG.md).
- Metric semantics and units live in [METRIC_SEMANTICS.md](METRIC_SEMANTICS.md).
- Tool/reference evidence surfaces live in [REFERENCES.md](REFERENCES.md).

## Examples
- Retention rates include numerator/denominator and units.

## Failure modes
- Missing retention context fails validation.

# Report Contract

## What
Defines the required contents of report.json and report bundles.

## Why
Guarantees reports are complete and comparable.

## Non-goals
- UI styling requirements.

## Contracts
- Reports must include toolchain versions, parameters, hashes, and metrics.
- aDNA VCF reports must include a `bias_audit` section with before/after damage-filter summary.
- `bias_audit` must include strategy id (`pmd_filter` or `ct_ga_masking`) and key deltas.
- VCF reports must include `coverage_regime` with:
  - `selected`
  - `thresholds_used`
  - `observed_coverage_stats`

## Examples
- FASTQ reports include trimming/retention metrics with units.

## Failure modes
- Missing sections cause report completeness failures.

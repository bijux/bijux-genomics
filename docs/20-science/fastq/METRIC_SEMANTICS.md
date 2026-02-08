# Metric Semantics (FASTQ)

## What
Defines the meaning, units, and interpretation constraints for FASTQ metrics.

## Why
Decision logic relies on stable metric semantics across planners, stages, and reports.

## Non-goals
- Explaining tool internals.
- Duplicating the metrics schema definitions.

## Contracts
- Metrics must declare numerator/denominator and units.
- Failure modes must be documented for metrics used in decisions.

## Examples
### retention
- numerator: reads_out
- denominator: reads_in
- units: reads
- failure modes: missing reads_in/out
- can be gamed by dropping low-quality reads without recording filters

### bases_kept
- numerator: bases_out
- denominator: bases_in
- units: bases

## Failure modes
- Unstated units or denominators lead to invalid comparisons.

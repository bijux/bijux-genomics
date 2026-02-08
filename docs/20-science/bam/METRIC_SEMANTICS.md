# Metric Semantics (BAM)

## What
Defines the meaning, units, and interpretation constraints for BAM metrics.

## Why
Downstream decisions and reports assume shared metric semantics; ambiguity leads to inconsistent scoring.

## Non-goals
- Replacing the metric schema definitions.
- Explaining every algorithm behind each metric.

## Contracts
- Metrics must include stable units and interpretation notes.
- Failure modes must be documented for each metric used in decisions.

## Examples
### damage_profile
- units: proportion
- failure modes: insufficient reads
- can be gamed by filtering damaged reads

### contamination_rate
- units: proportion
- failure modes: low coverage
- can be gamed by selective alignment

## Failure modes
- Ambiguous units or scaling cause incompatible comparisons.

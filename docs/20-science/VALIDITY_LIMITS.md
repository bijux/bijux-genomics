# Scientific Validity Limits

## What
Defines the scientific claims Bijux does not make and the limits on interpretation.

## Why
Users need explicit limits to avoid overconfidence in downstream decisions.

## Non-goals
- Replacing domain-specific interpretation guides.
- Providing probabilistic guarantees beyond what metrics support.

## Contracts
- Limits must be stated alongside reports and metric definitions.
- Assumptions must be traceable to inputs and defaults.

## Examples
### What Bijux does not claim
- Authentication is probabilistic, not absolute.
- Contamination estimates depend on model assumptions.
- Damage metrics are sensitive to coverage and filtering.

### How assumptions are exposed
- Reports include method assumptions and metrics context.
- Defaults ledger records parameter choices.

## Failure modes
- Omitted limits lead to invalid scientific conclusions.

## eDNA and Pollen Limits
### Database bias
- Marker databases are incomplete and taxonomically uneven.
- Absence in outputs does not prove biological absence.

### Marker choice
- Different markers resolve different taxonomic depths.
- Cross-marker comparisons require explicit normalization and caveats.

### Compositionality
- Read counts are compositional proxies, not absolute abundance.
- Relative abundance shifts may reflect library effects, not ecology alone.

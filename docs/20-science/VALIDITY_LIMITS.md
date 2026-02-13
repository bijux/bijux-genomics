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

## VCF Downstream Demography Limits
### Population structure
- PCA/clustering outputs depend on LD pruning, missingness thresholds, and cohort composition.
- Admixture-like assignments are model summaries, not hard ancestry labels.

### ROH and IBD
- ROH and IBD metrics are sensitive to marker density, phasing quality, and genotype error.
- Cross-tool comparison is valid only under the same metrics schema and compatible parameterization.

### Demography
- Recent Ne summaries are model-dependent and require explicit generation-time/recombination assumptions.
- Demography estimates should be interpreted with confidence intervals and assumption flags.

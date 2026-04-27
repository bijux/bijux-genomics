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
- Assumptions must be traceable to inputs and defaults recorded in [Scientific Defaults](SCIENTIFIC_DEFAULTS.md).

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

## FASTQ Limits
- Authority: [FASTQ Scientific Spec](fastq/SCIENTIFIC_SPEC.md).
- Read retention, trimming, and merge metrics are only comparable when layout, adapter policy, minimum lengths, and upstream asset identities stay fixed.
- Taxonomy-screening outputs remain database-bound classifier summaries, not direct abundance truths.
- Amplicon-only stages such as ASV inference, OTU clustering, and chimera removal must not be interpreted as shotgun-general defaults.

## BAM Limits
- Authority: [BAM Methodological Intent](bam/METHODOLOGICAL_INTENT.md).
- Authentication is probabilistic, not absolute.
- Contamination estimates depend on model assumptions and reference authority.
- Damage metrics are sensitive to coverage, UDG treatment, and filtering.

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
- Authority: [VCF Population Structure](vcf/POPULATION_STRUCTURE.md).
- PCA/clustering outputs depend on LD pruning, missingness thresholds, and cohort composition.
- Admixture-like assignments are model summaries, not hard ancestry labels.

### ROH and IBD
- Authorities: [VCF ROH](vcf/ROH.md) and [VCF IBD](vcf/IBD.md).
- ROH and IBD metrics are sensitive to marker density, phasing quality, and genotype error.
- Cross-tool comparison is valid only under the same metrics schema and compatible parameterization.

### Demography
- Authority: [VCF Demography](vcf/DEMOGRAPHY.md).
- Recent Ne summaries are model-dependent and require explicit generation-time/recombination assumptions.
- Demography estimates should be interpreted with confidence intervals and assumption flags.

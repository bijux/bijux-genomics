# VCF Stage Taxonomy (Downstream Placeholder Baseline)

Purpose: define minimum viable downstream VCF stage taxonomy used by planned expansion.

Scope: contract placeholder taxonomy for `vcf` stages beyond call/filter/stats.

Contracts:
- Planned stages must remain explicit in domain stage files and CI stage registries.
- Each stage must declare inputs/outputs/defaults_source/compatible_tools.

Minimum taxonomy:
- `vcf.qc`: quality gate and normalization checks.
- `vcf.pca`: population structure projection features.
- `vcf.admixture`: ancestry mixture estimation features.
- `vcf.ibd`: identity-by-descent segment inference inputs/outputs.
- `vcf.phasing`: haplotype phasing preparation/execution.
- `vcf.imputation`: post-phasing imputation staging.

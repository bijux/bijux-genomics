# VCF Population Structure Stage

## Purpose
Define the governed population-structure stage family for VCF cohorts without collapsing distinct QC, PCA, admixture, and summary boundaries into one generic narrative.

## Scope
This science surface covers:
- `vcf.qc` for cohort-level missingness and MAF gating before structure inference.
- `vcf.pca` for principal-component summaries from the admitted structure-tool family.
- `vcf.admixture` for mixture-style downstream summaries from the admitted PLINK-family cohort-analysis surface.
- `vcf.population_structure` for the higher-level report contract that rolls consumed PCA and admixture evidence into a governed summary.

## Non-goals
- Replacing full study design or interpretation of ancestry history.
- Pretending that PCA, admixture, and final structure reporting are interchangeable stages.

## Contracts
- `vcf.qc` emits the governed cohort-QC report contract before downstream inference is interpreted.
- `vcf.pca` emits PCA-oriented summaries from the admitted `plink2` and `eigensoft` tool family.
- `vcf.admixture` emits governed cluster-fraction summaries on the admitted `plink` and `plink2` matrix-tool surface.
- `vcf.population_structure` emits `population_structure_report` with schema `bijux.vcf.population_structure.v1`.
- Required `vcf.population_structure` metrics include sample count, pair count, within-population pair count, and cross-population pair count.
- Governed defaults currently stay `plink2` for `vcf.qc`, `vcf.pca`, `vcf.admixture`, and `vcf.population_structure`.

## Validity Limits
- LD pruning, missingness filtering, and cohort composition materially change `vcf.pca`, `vcf.admixture`, and `vcf.population_structure` outputs.
- Cross-run comparison requires fixed build, fixed admitted tool versions, and unchanged QC thresholds.
- Structure summaries are model-dependent reports, not absolute ancestry truths.

# VCF Population Structure Stage

## Purpose
Define methodological intent for `vcf.population_structure` outputs used in downstream inference.

## Scope
Applies to PCA/cluster summary generation from filtered VCF cohorts.

## Non-goals
- Replacing full population genetics study design.

## Contracts
- Stage contract: `domain/vcf/stages/population_structure.yaml`.
- Expected output: `population_structure_report`.
- Baseline planned tools: `plink`, `plink2`, `eigensoft`.
- Output contract requires `metrics.json` with schema `bijux.vcf.population_structure.v1`.
- Required metrics include PCA variance, PC axes, cluster assignments, and admixture model selection summaries.

## Validity Limits
- Sensitive to LD pruning choices and sample composition.
- Cross-run comparison requires fixed panel/build and pinned tool versions.
- Clustering/admixture summaries are model-dependent and not absolute ancestry truths.

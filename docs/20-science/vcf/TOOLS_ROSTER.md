# VCF Tools Roster

## What
Manifest-backed roster of admitted tools for every VCF stage.

## Why
The VCF science layer needs a readable tool inventory that mirrors the stage manifests instead of burying tool admission inside generated registries.

## Non-goals
- Ranking tools by scientific quality.
- Declaring comparability between different downstream inference methods.

## Contracts
- Every VCF stage must appear exactly once.
- `Status` must mirror `domain/vcf/stages/*.yaml`.
- `Admitted tools` must stay within each stage manifest's `compatible_tools`.

| Stage | Status | Admitted tools | Rationale |
| --- | --- | --- | --- |
| vcf.call | supported | bcftools | The current governed call surface stays intentionally single-tool and deterministic. |
| vcf.call_diploid | supported | bcftools | Diploid baseline remains the currently governed production call surface. |
| vcf.call_gl | supported | angsd, bcftools | GL-first calling keeps both the current governed baseline and the planned low-coverage alternative visible. |
| vcf.call_pseudohaploid | supported | angsd, bcftools | Pseudohaploid calling keeps the low-coverage alternative explicit without hiding the current baseline. |
| vcf.damage_filter | supported | bcftools, angsd | Damage-aware filtering remains coupled to GL-aware evidence families. |
| vcf.filter | supported | bcftools | Deterministic filtering is still a single admitted normalization path. |
| vcf.gl_propagation | supported | bcftools, angsd | GL/PL retention is governed across both the baseline and planned low-coverage tool families. |
| vcf.stats | supported | bcftools | Required VCF summary metrics remain bound to the current production baseline. |
| vcf.qc | planned | plink, plink2 | Cohort-level QC stays planned until the downstream VCF surface is promoted. |
| vcf.pca | planned | plink2, eigensoft | PCA support stays comparative across the two admitted structure-analysis families. |
| vcf.admixture | planned | plink, plink2 | Admixture-oriented staging remains planned and anchored to the current admitted matrix-tool surface. |
| vcf.population_structure | planned | plink, plink2, eigensoft | Population-structure summaries admit both PLINK-family and EIGENSOFT-family tooling. |
| vcf.phasing | planned | beagle, shapeit5, eagle | Phasing remains a planned multi-backend comparative surface. |
| vcf.prepare_reference_panel | supported | bcftools | Reference-panel preparation stays deterministic even while downstream imputation remains planned. |
| vcf.imputation_metrics | planned | beagle, glimpse, impute5, minimac4 | Imputation-metrics admission stays broad while promotion evidence is still incomplete. |
| vcf.impute | planned | beagle, glimpse, impute5, minimac4 | Explicit imputation execution mirrors the admitted imputation family. |
| vcf.postprocess | planned | bcftools | Post-imputation normalization remains deterministic and single-tool. |
| vcf.ibd | planned | germline, ibdhap | IBD remains a comparative multi-backend planned surface. |
| vcf.roh | planned | plink2 | ROH remains a planned but deterministic single-tool surface. |
| vcf.demography | planned | ibdne | Demography currently stays coupled to the admitted IBDNe-style downstream summary path. |

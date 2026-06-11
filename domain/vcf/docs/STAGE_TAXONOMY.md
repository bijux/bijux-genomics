# VCF Stage Taxonomy

## Purpose
Define the governed stage-grouping vocabulary for the VCF domain.

## Scope
Covers every `domain/vcf/stages/*.yaml` entry, including supported calling stages and planned downstream analysis stages.

## Contracts
- Every VCF stage manifest must appear exactly once here.
- `Status` must mirror the stage manifest.
- `Phase` explains where the stage sits in the VCF workflow; `Class` explains what kind of boundary it represents.

| Stage | Phase | Class | Status | Intent |
| --- | --- | --- | --- | --- |
| vcf.call | calling | mutation | supported | Emit the current deterministic baseline VCF call surface. |
| vcf.call_diploid | calling | mutation | supported | Emit diploid genotype calls for high-confidence cohorts. |
| vcf.call_gl | calling | mutation | supported | Emit genotype-likelihood-first outputs for low-coverage and aDNA-aware workflows. |
| vcf.call_pseudohaploid | calling | mutation | supported | Emit one-allele representations for low-coverage contexts. |
| vcf.damage_filter | damage mediation | mutation | supported | Apply damage-aware masking or filtering before downstream inference. |
| vcf.filter | normalization | mutation | supported | Apply deterministic VCF filter normalization. |
| vcf.gl_propagation | provenance retention | mutation | supported | Preserve GL/PL evidence across downstream transforms. |
| vcf.stats | reporting | report | supported | Emit required summary metrics for quality review. |
| vcf.qc | downstream gating | report | supported | Apply cohort-level QC summaries and thresholds. |
| vcf.pca | downstream inference | inference | supported | Estimate PCA-based structure projections with normalized per-sample component output. |
| vcf.admixture | downstream inference | inference | supported | Estimate ancestry-mixture-style summaries. |
| vcf.population_structure | downstream inference | inference | supported | Emit broader structure summaries from consumed PCA and admixture evidence. |
| vcf.phasing | panel mediation | mutation | supported | Phase haplotypes before imputation or IBD. |
| vcf.prepare_reference_panel | panel mediation | mutation | supported | Normalize and prepare reference panels. |
| vcf.imputation_metrics | panel mediation | report | supported | Summarize imputation-quality evidence from governed imputation outputs. |
| vcf.impute | panel mediation | mutation | supported | Execute explicit imputation with a pinned backend. |
| vcf.postprocess | normalization | mutation | supported | Normalize INFO/FILTER/FORMAT surfaces after imputation with the governed deterministic bcftools baseline. |
| vcf.ibd | downstream inference | inference | supported | Estimate pairwise IBD segments with normalized pair rows and explicit insufficient-overlap status. |
| vcf.roh | downstream inference | inference | supported | Estimate runs of homozygosity burden and normalized segment output. |
| vcf.demography | downstream inference | inference | planned | Estimate recent demography summaries from IBD-derived evidence. |

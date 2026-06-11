# VCF Default Settings (Contract Baseline)

Purpose: define deterministic blessed defaults and rationale for each VCF stage.

## Inputs
- `vcf.call`: aligned evidence represented as VCF-ready records.
- `vcf.filter`: raw called VCF from `vcf.call`.
- `vcf.stats`: filtered VCF from `vcf.filter`.
- `vcf.qc`: filtered/stats-enriched VCF bundle.
- `vcf.pca`: LD-pruned VCF matrix.
- `vcf.admixture`: VCF matrix prepared for ancestry decomposition.
- `vcf.ibd`: phased/imputed-compatible VCF plus sample metadata.
- `vcf.phasing`: filtered VCF + reference panels (when enabled).
- `vcf.imputation_metrics`: phased VCF + panel metadata + imputation QC evidence.
- `vcf.impute`: phased VCF + panel metadata.
- `vcf.postprocess`: imputed VCF + report filters.
- `vcf.prepare_reference_panel`: raw panel VCF/BCF + build metadata.

## Outputs
- `vcf.call` -> `called_vcf`
- `vcf.filter` -> `filtered_vcf`
- `vcf.stats` -> `stats_json`
- `vcf.qc` -> `qc_report`
- `vcf.pca` -> `pca_report`
- `vcf.admixture` -> `admixture_report`
- `vcf.population_structure` -> `population_structure_report`
- `vcf.ibd` -> `ibd_segments`
- `vcf.phasing` -> `phased_vcf`
- `vcf.imputation_metrics` -> `imputation_metrics_json`
- `vcf.impute` -> `imputed_vcf`
- `vcf.postprocess` -> `postprocess_vcf`
- `vcf.prepare_reference_panel` -> `prepared_panel`

## Key Parameters
- calling strictness and emit mode (`vcf.call`)
- filter expression policy (`vcf.filter`)
- summary aggregation mode (`vcf.stats`)
- missingness/maf guardrails (`vcf.qc`, `vcf.pca`, `vcf.admixture`)
- window/segment constraints (`vcf.ibd`)
- panel and phasing algorithm toggles (`vcf.phasing`, `vcf.imputation_metrics`)
- panel and imputation engine toggles (`vcf.impute`)
- post-imputation INFO/filter normalization toggles (`vcf.postprocess`)
- panel normalization/index strategy (`vcf.prepare_reference_panel`)

## Validity Limits
- Defaults are valid only with pinned production/approved planned tool versions.
- Reference build must remain explicit and unchanged for comparability.
- Stage ordering and contract IO keys must remain schema-compatible.

## Blessed Defaults And Rationale
- `vcf.call` default: `bcftools`. rationale: deterministic baseline caller for current production profile.
- `vcf.filter` default: `bcftools`. rationale: stable filtering semantics for regression comparability.
- `vcf.stats` default: `bcftools`. rationale: minimal required metrics for quality gating.
- `vcf.qc` default: `plink2`. rationale: the governed cohort-QC baseline keeps missingness, MAF, and HWE summaries anchored to the admitted PLINK-family execution surface while retaining deterministic bcftools compatibility for comparison.
- `vcf.pca` default: `plink2`. rationale: current governed PCA default keeps sample-complete eigenvector output, eigenvalue reporting, and metadata joins anchored to the PLINK2 smoke-backed cohort path while retained `eigensoft` stays comparative.
- `vcf.admixture` default: `plink2`. rationale: the governed admixture baseline keeps cohort preparation, K selection, and normalized cluster fractions anchored to the admitted PLINK-family execution surface.
- `vcf.phasing` default: `shapeit5`. rationale: current governed production default keeps phasing anchored to the dedicated phasing backend while Beagle and Eagle stay comparative.
- `vcf.imputation_metrics` default: `beagle`. rationale: keep the governed imputation-metrics contract anchored to the same backend family that produces the source imputation evidence.
- `vcf.impute` default: `beagle`. rationale: current governed production default keeps explicit panel-aware imputation runnable while retained alternatives stay comparative.
- `vcf.postprocess` default: `bcftools`. rationale: deterministic normalization/filter baseline for governed post-imputation outputs.
- `vcf.prepare_reference_panel` default: `bcftools`. rationale: deterministic reference panel prep baseline.
- `vcf.call_gl` default: `bcftools`. rationale: current governed production default keeps GL emission runnable while `angsd` remains a planned low-coverage alternative.
- `vcf.call_diploid` default: `bcftools`. rationale: deterministic diploid baseline for the current governed production profile.
- `vcf.call_pseudohaploid` default: `bcftools`. rationale: current governed production default preserves the pseudohaploid contract while `angsd` remains a planned alternative.
- `vcf.damage_filter` default: `bcftools`. rationale: deterministic PMD/C>T-G>A masking contract anchor for the current governed profile.
- `vcf.gl_propagation` default: `bcftools`. rationale: preserves GL fields across downstream handoffs in the current governed profile.
- `vcf.population_structure` default: `plink2`. rationale: the governed population-structure baseline keeps consumed PCA and admixture evidence on the admitted PLINK-family structure path with explicit sample-group distance summaries.
- `vcf.ibd` default: `germline` (planned). rationale: the active normalized pairwise-segment contract remains pinned to the germline family surface while external runtime packaging is still pending.
- `vcf.roh` default: `plink2`. rationale: deterministic ROH interval extraction with normalized segment and per-sample summaries.
- `vcf.demography` default: `ibdne` (planned). rationale: the active deterministic Ne summary contract mirrors the IBDNe surface while external runtime packaging is still pending.

single_tool_justification: vcf.call
single_tool_justification: vcf.filter
single_tool_justification: vcf.stats
single_tool_justification: vcf.qc
single_tool_justification: vcf.pca
single_tool_justification: vcf.admixture
single_tool_justification: vcf.ibd
single_tool_justification: vcf.phasing
single_tool_justification: vcf.imputation_metrics
single_tool_justification: vcf.impute
single_tool_justification: vcf.postprocess
single_tool_justification: vcf.prepare_reference_panel
single_tool_justification: vcf.call_diploid
single_tool_justification: vcf.ibd
single_tool_justification: vcf.roh
single_tool_justification: vcf.demography

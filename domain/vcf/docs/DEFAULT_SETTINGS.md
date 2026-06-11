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
- `vcf.qc` -> `qc_json`
- `vcf.pca` -> `pca_json`
- `vcf.admixture` -> `admixture_json`
- `vcf.ibd` -> `ibd_json`
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
- `vcf.qc` default: `plink2` (planned). rationale: keep planned cohort-QC behavior on an admitted downstream matrix tool instead of inventing a VCF-only placeholder.
- `vcf.pca` default: `plink2` (planned). rationale: current planned PCA baseline stays aligned with the governed population-structure tooling family.
- `vcf.admixture` default: `plink2` (planned). rationale: keep the planned admixture surface anchored to an admitted matrix-preparation backend until a dedicated admixture tool is formally admitted.
- `vcf.ibd` default: `germline` (planned). rationale: current planned IBD baseline names the intended segment caller while alternative tools stay comparative.
- `vcf.phasing` default: `shapeit5`. rationale: current governed production default tracks the modern phasing backend while retained alternatives stay comparative.
- `vcf.imputation_metrics` default: `beagle` (planned). rationale: keep the planned imputation-metrics contract anchored to the same governed backend family that produces the source imputation evidence.
- `vcf.impute` default: `beagle`. rationale: current governed production default keeps explicit imputation runnable while retained alternatives stay comparative.
- `vcf.postprocess` default: `bcftools`. rationale: deterministic normalization/filter baseline for governed post-imputation outputs.
- `vcf.prepare_reference_panel` default: `bcftools`. rationale: deterministic reference panel prep baseline.
- `vcf.call_gl` default: `bcftools`. rationale: current governed production default keeps GL emission runnable while `angsd` remains a planned low-coverage alternative.
- `vcf.call_diploid` default: `bcftools`. rationale: deterministic diploid baseline for the current governed production profile.
- `vcf.call_pseudohaploid` default: `bcftools`. rationale: current governed production default preserves the pseudohaploid contract while `angsd` remains a planned alternative.
- `vcf.damage_filter` default: `bcftools`. rationale: deterministic PMD/C>T-G>A masking contract anchor for the current governed profile.
- `vcf.gl_propagation` default: `bcftools`. rationale: preserves GL fields across downstream handoffs in the current governed profile.
- `vcf.population_structure` default: `plink2` (planned). rationale: deterministic PCA/pop-structure baseline with stable metrics schema.
- `vcf.roh` default: `plink2` (planned). rationale: deterministic ROH interval extraction and summary bins.
- `vcf.demography` default: `ibdne` (planned). rationale: deterministic effective population size summary from IBD-derived inputs.

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
single_tool_justification: vcf.roh
single_tool_justification: vcf.demography

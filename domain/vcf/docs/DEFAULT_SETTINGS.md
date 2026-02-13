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
- `vcf.imputation`: phased VCF + panel metadata.

## Outputs
- `vcf.call` -> `called_vcf`
- `vcf.filter` -> `filtered_vcf`
- `vcf.stats` -> `stats_json`
- `vcf.qc` -> `qc_json`
- `vcf.pca` -> `pca_json`
- `vcf.admixture` -> `admixture_json`
- `vcf.ibd` -> `ibd_json`
- `vcf.phasing` -> `phased_vcf`
- `vcf.imputation` -> `imputed_vcf`

## Key Parameters
- calling strictness and emit mode (`vcf.call`)
- filter expression policy (`vcf.filter`)
- summary aggregation mode (`vcf.stats`)
- missingness/maf guardrails (`vcf.qc`, `vcf.pca`, `vcf.admixture`)
- window/segment constraints (`vcf.ibd`)
- panel and phasing algorithm toggles (`vcf.phasing`, `vcf.imputation`)

## Validity Limits
- Defaults are valid only with pinned production/approved planned tool versions.
- Reference build must remain explicit and unchanged for comparability.
- Stage ordering and contract IO keys must remain schema-compatible.

## Blessed Defaults And Rationale
- `vcf.call` default: `bcftools`. rationale: deterministic baseline caller for current production profile.
- `vcf.filter` default: `bcftools`. rationale: stable filtering semantics for regression comparability.
- `vcf.stats` default: `bcftools`. rationale: minimal required metrics for quality gating.
- `vcf.qc` default: `bcftools` (planned). rationale: keep planned stage deterministic until downstream tools are promoted.
- `vcf.pca` default: `bcftools` (planned placeholder). rationale: placeholder baseline while `plink/plink2/eigensoft` admission is in progress.
- `vcf.admixture` default: `bcftools` (planned placeholder). rationale: preserves deterministic contract while candidate tools are evaluated.
- `vcf.ibd` default: `bcftools` (planned placeholder). rationale: placeholder contract baseline before `germline/ibdseq/ibdhap/ibdne` promotion.
- `vcf.phasing` default: `bcftools` (planned placeholder). rationale: deterministic staging until `beagle/shapeit` policy promotion.
- `vcf.imputation` default: `bcftools` (planned placeholder). rationale: deterministic staging until imputation toolchain is admitted.

single_tool_justification: vcf.call
single_tool_justification: vcf.filter
single_tool_justification: vcf.stats
single_tool_justification: vcf.qc
single_tool_justification: vcf.pca
single_tool_justification: vcf.admixture
single_tool_justification: vcf.ibd
single_tool_justification: vcf.phasing
single_tool_justification: vcf.imputation

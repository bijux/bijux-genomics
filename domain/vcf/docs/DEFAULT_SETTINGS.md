# VCF Default Settings (Contract Baseline)

Purpose: define deterministic default behavior for each VCF stage.

## Inputs
- `vcf.call`: aligned evidence and reference context represented as VCF-ready input.
- `vcf.filter`: raw called VCF.
- `vcf.stats`: filtered VCF.

## Outputs
- `vcf.call`: `called_vcf`.
- `vcf.filter`: `filtered_vcf`.
- `vcf.stats`: `vcf_stats`.

## Key Parameters
- calling strictness threshold
- filter expression policy
- stats aggregation mode

## Validity Limits
- only pinned tool versions are valid
- reference build and caller mode must stay fixed for comparability
- stage input/output contracts must remain schema-compatible

## Stage Defaults
- `vcf.call`: default `bcftools`.
- `vcf.filter`: default `bcftools`.
- `vcf.stats`: default `bcftools`.

single_tool_justification: vcf.call
single_tool_justification: vcf.filter
single_tool_justification: vcf.stats

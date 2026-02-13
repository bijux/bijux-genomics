# VCF Default Settings (Contract Baseline)

Purpose: define deterministic default behavior for each VCF stage.

## Inputs Per Stage
- `vcf.call`
  input contract: aligned evidence represented as VCF-ready records.
- `vcf.filter`
  input contract: raw called VCF produced by `vcf.call`.
- `vcf.stats`
  input contract: filtered VCF produced by `vcf.filter`.

## Outputs Per Stage
- `vcf.call` output: `called_vcf`.
- `vcf.filter` output: `filtered_vcf`.
- `vcf.stats` output: `stats_json`.

## Invariants
- pinned tool versions only; no floating versions.
- reference build and stage defaults stay fixed for comparability.
- stage input/output keys remain schema-compatible across updates.

## Key Parameters
- calling strictness threshold for `vcf.call`.
- filter expression policy for `vcf.filter`.
- stats aggregation mode for `vcf.stats`.

## Validity Limits
- defaults are valid only with pinned production tool versions.
- defaults assume schema-compatible input/output contracts between stages.
- defaults require deterministic reference context and stage ordering.

## Default Parameters Rationale
- `vcf.call`
  rationale: prioritize deterministic and broadly accepted calling defaults for baseline comparisons.
- `vcf.filter`
  rationale: preserve high-confidence variants with stable filters suitable for regression checks.
- `vcf.stats`
  rationale: keep summary metrics minimal, reproducible, and comparable across runs.

## Default Tool Selection
- `vcf.call`: default `bcftools` (single-tool justification recorded in `domain/vcf/stages/call.yaml`).
- `vcf.filter`: default `bcftools` (single-tool justification recorded in `domain/vcf/stages/filter.yaml`).
- `vcf.stats`: default `bcftools` (single-tool justification recorded in `domain/vcf/stages/stats.yaml`).

single_tool_justification: vcf.call
single_tool_justification: vcf.filter
single_tool_justification: vcf.stats

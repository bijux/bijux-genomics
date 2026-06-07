# VCF Downstream Compatibility Matrix

## Purpose
Defines generated compatibility rows for species/build/panel/tool combinations used by VCF downstream planning.

## Scope
Generated from panel compatibility tags and VCF downstream tool registry stage mappings.

## Non-goals
- Proving runtime availability of tools or panel materialization state.

## Contracts
- The matrix is generated authority and must stay in sync with panel and registry sources.
- Missing expected combinations are treated as compatibility governance drift.

Generated from `configs/vcf/panels/panels.toml` and `configs/ci/registry/tool_registry_vcf_downstream.toml`.

| species_id | build_id | panel_id | tool_id | stage_ids |
|---|---|---|---|---|
| Homo sapiens | GRCh38 | hsapiens_grch38_full | beagle | vcf.phasing |
| Homo sapiens | GRCh38 | hsapiens_grch38_full | eagle | vcf.phasing |
| Homo sapiens | GRCh38 | hsapiens_grch38_full | glimpse | vcf.impute, vcf.imputation_metrics |
| Homo sapiens | GRCh38 | hsapiens_grch38_full | impute5 | vcf.impute, vcf.imputation_metrics |
| Homo sapiens | GRCh38 | hsapiens_grch38_full | minimac4 | vcf.impute, vcf.imputation_metrics |
| Homo sapiens | GRCh38 | hsapiens_grch38_full | shapeit5 | vcf.phasing |
| Homo sapiens | GRCh38 | hsapiens_grch38_mini | beagle | vcf.phasing |
| Homo sapiens | GRCh38 | hsapiens_grch38_mini | eagle | vcf.phasing |
| Homo sapiens | GRCh38 | hsapiens_grch38_mini | glimpse | vcf.impute, vcf.imputation_metrics |
| Homo sapiens | GRCh38 | hsapiens_grch38_mini | impute5 | vcf.impute, vcf.imputation_metrics |
| Homo sapiens | GRCh38 | hsapiens_grch38_mini | minimac4 | vcf.impute, vcf.imputation_metrics |
| Homo sapiens | GRCh38 | hsapiens_grch38_mini | shapeit5 | vcf.phasing |

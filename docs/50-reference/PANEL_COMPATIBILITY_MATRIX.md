<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: scripts/tooling/generate-panel-compatibility-matrix.sh -->

# PANEL_COMPATIBILITY_MATRIX

## Purpose
Defines generated compatibility coverage for species/build, panel/map pairs, and downstream tool backends.

## Scope
Derived from panel and map catalogs to document declared tool-tag compatibility.

## Non-goals
- Replacing stage-level validation or runtime compatibility checks.

## Contracts
- Matrix rows are generated from catalog authority and must not be hand-edited.
- Missing species/build map entries must be represented explicitly as unsupported rows.

| Species | Build | Panel ID | Map ID | Tool Backend | Supported | Notes |
|---|---|---|---|---|---|---|
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_full` | `hsapiens_grch38_chr_map` | `bcftools` | `yes` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_full` | `hsapiens_grch38_chr_map` | `beagle` | `no` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_full` | `hsapiens_grch38_chr_map` | `eagle` | `yes` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_full` | `hsapiens_grch38_chr_map` | `glimpse` | `yes` | GLIMPSE format=bcf+sites |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_full` | `hsapiens_grch38_chr_map` | `impute5` | `yes` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_full` | `hsapiens_grch38_chr_map` | `minimac4` | `yes` | requires panel m3vcf |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_full` | `hsapiens_grch38_chr_map` | `shapeit5` | `yes` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_mini` | `hsapiens_grch38_chr_map` | `bcftools` | `yes` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_mini` | `hsapiens_grch38_chr_map` | `beagle` | `no` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_mini` | `hsapiens_grch38_chr_map` | `eagle` | `yes` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_mini` | `hsapiens_grch38_chr_map` | `glimpse` | `yes` | GLIMPSE format=bcf+sites |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_mini` | `hsapiens_grch38_chr_map` | `impute5` | `yes` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_mini` | `hsapiens_grch38_chr_map` | `minimac4` | `yes` | requires panel m3vcf |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_mini` | `hsapiens_grch38_chr_map` | `shapeit5` | `yes` | - |

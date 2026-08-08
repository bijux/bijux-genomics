<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- tooling run generate-panel-compatibility-matrix -->

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
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_1kg_3202_chr22` | `hsapiens_grch38_glimpse_chr22` | `bcftools` | `no` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_1kg_3202_chr22` | `hsapiens_grch38_glimpse_chr22` | `beagle` | `no` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_1kg_3202_chr22` | `hsapiens_grch38_glimpse_chr22` | `eagle` | `no` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_1kg_3202_chr22` | `hsapiens_grch38_glimpse_chr22` | `glimpse` | `yes` | GLIMPSE format=bcf+sites |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_1kg_3202_chr22` | `hsapiens_grch38_glimpse_chr22` | `impute5` | `yes` | - |
| `Homo sapiens` | `GRCh38` | `hsapiens_grch38_1kg_3202_chr22` | `hsapiens_grch38_glimpse_chr22` | `shapeit5` | `yes` | - |

# Damage-Aware Genotype Logic

## Scope

This document defines the VCF aDNA damage-aware path used by:

- `vcf.call_gl`
- `vcf.damage_filter`
- `vcf.impute` (GLIMPSE-oriented low-coverage path)

## Required Contracts

`vcf.call_gl` and `vcf.damage_filter` share a compatible evidence contract:

- FORMAT evidence: at least one of `GL`, `GP`, `PL` when available.
- INFO evidence: any of `CT_GA_DAMAGE_RATIO`, `DEAM5P`, `DEAM3P`, `PMD_SCORE` (or `PMD`/`PMDSCORE`).

If both are absent, `vcf.damage_filter` falls back to proxy transition heuristics and emits warning code:

- `W_VCF_DAMAGE_FILTER_PROXY_ONLY`

## Filtering and Masking

`vcf.damage_filter` computes:

- C>T / G>A asymmetry.
- Read-position stratified signal (5' vs 3').
- PMD-aware filtering when PMD fields are present.

Masking behavior is configurable:

- `BIJUX_VCF_DAMAGE_MASK_MODE=remove` (default): remove filtered transition records.
- `BIJUX_VCF_DAMAGE_MASK_MODE=mark`: keep records and mark with `LOWCONF_DAMAGE_TERMINAL`.

Terminal threshold:

- `BIJUX_VCF_DAMAGE_TERMINAL_THRESHOLD` (default `0.50`)
- For `BIJUX_LIBRARY_TYPE=ssdna`, threshold is tightened for stronger terminal damage expectations.

## UDG + Library Effects

- UDG regime influences effective damage thresholds.
- Library type (`ssdna` vs `dsdna`) is included in manifest/provenance and affects terminal expectations.

## Artifacts

`vcf.damage_filter` emits:

- `damage_filter_summary.json`
- `damage_filter_counts.json`
- `warnings.json`
- `damage_genotype_manifest.json`

`warnings.json` must include explicit "what filtered and why" breakdown.

## Refusal / Gate Examples

- If `BIJUX_ADNA_MODE=1` and `vcf.damage_filter` is not scheduled, pipeline refuses unless:
  - `BIJUX_ALLOW_SKIP_DAMAGE_FILTER=1`
- Ancient QC defaults skip modern-only HWE metrics unless explicitly enabled.

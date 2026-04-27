# Damage-Aware Genotype Logic

## Scope

This document defines the VCF aDNA damage-aware path used by:

- `vcf.call_gl`
- `vcf.call_pseudohaploid`
- `vcf.damage_filter`
- `vcf.gl_propagation`
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

## Accept Examples

- `udg_regime=non_udg`, `library_type=ssdna`, and stage input includes `GL` plus any damage INFO tags:
  - Stage runs with terminal-aware thresholds and emits `damage_genotype_manifest.json`.
- `udg_regime=udg`, `library_type=dsdna`, with PMD absent:
  - Stage still runs, records proxy usage in warnings when required fields are missing, and keeps deterministic outputs.

## Post-Impute Residual Check

- Downstream imputation stage writes residual transition asymmetry diagnostics and warnings so damage signatures can be re-audited after imputation merge.

## Purpose

Keep the damage-aware VCF path explicit across calling, damage filtering, GL retention, and low-coverage imputation so ancient-DNA runs do not silently collapse into modern-diploid assumptions.

## Non-goals

- Declaring one damage model universally best across all library types.
- Replacing lower-level tool parameter ledgers or provenance artifacts.

## Contracts

- `vcf.call_gl` and `vcf.call_pseudohaploid` must emit enough provenance for later damage-sensitive interpretation.
- `vcf.damage_filter` must state whether it used explicit PMD/damage evidence or proxy heuristics.
- `vcf.gl_propagation` must preserve likelihood-bearing fields needed by low-coverage downstream stages.
- `vcf.impute` must record residual damage diagnostics when it consumes the damage-aware calling path.

# VCF Stage Catalog

## What
Canonical stage definitions for the currently governed VCF execution surface.

## Why
VCF stage names carry scientific meaning. This file keeps supported execution boundaries explicit so downstream science docs do not drift into tool- or stage-invented terminology.

## Non-goals
- Exhaustive post-VCF roadmap coverage in one pass.
- Replacing the lower-level stage manifests under `domain/vcf/stages/`.

## Contracts
- Every documented stage entry must declare purpose, inputs/outputs, metrics, tools, defaults, and references.
- This initial catalog covers the supported VCF runtime surface; planned stages stay visible elsewhere until their catalog entries are filled in.

### vcf.call {#vcf-call}
- Status: supported.
- Purpose: emit the deterministic baseline VCF call surface used by the current governed runtime.
- Inputs/Outputs: aligned evidence → called VCF.
- Metrics: called site count, filtered site count.
- Tools: bcftools.
- Defaults: default `bcftools`; rationale lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/call.yaml`, `domain/vcf/index.yaml`.

### vcf.call_diploid {#vcf-call-diploid}
- Status: supported.
- Purpose: emit diploid genotype calls for high-confidence modern-DNA style cohorts.
- Inputs/Outputs: aligned evidence → diploid VCF.
- Metrics: diploid call count, genotype completeness.
- Tools: bcftools.
- Defaults: default `bcftools`; rationale lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/call_diploid.yaml`, `domain/vcf/index.yaml`.

### vcf.call_gl {#vcf-call-gl}
- Status: supported.
- Purpose: emit genotype-likelihood-first outputs for low-coverage and aDNA-aware workflows.
- Inputs/Outputs: aligned evidence → GL-oriented VCF.
- Metrics: GL-emitting site count, retained likelihood fields.
- Tools: angsd, bcftools.
- Defaults: default `bcftools`; rationale lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/call_gl.yaml`, `docs/20-science/vcf/DAMAGE_AWARE_GENOTYPE_LOGIC.md`.

### vcf.call_pseudohaploid {#vcf-call-pseudohaploid}
- Status: supported.
- Purpose: emit one-allele representations for low-coverage contexts where diploid calls are unstable.
- Inputs/Outputs: aligned evidence → pseudohaploid VCF.
- Metrics: pseudohaploid site count, retained allele count.
- Tools: angsd, bcftools.
- Defaults: default `bcftools`; rationale lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/call_pseudohaploid.yaml`, `docs/20-science/vcf/ROADMAP.md`.

### vcf.damage_filter {#vcf-damage-filter}
- Status: supported.
- Purpose: apply transition-aware and PMD-aware damage filters before downstream inference.
- Inputs/Outputs: VCF with damage evidence → damage-filtered VCF.
- Metrics: filtered transition count, proxy-warning count.
- Tools: bcftools, angsd.
- Defaults: default `bcftools`; rationale lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/damage_filter.yaml`, `docs/20-science/vcf/DAMAGE_AWARE_GENOTYPE_LOGIC.md`.

### vcf.filter {#vcf-filter}
- Status: supported.
- Purpose: apply deterministic pass/filter normalization to called VCF records.
- Inputs/Outputs: raw called VCF → filtered VCF.
- Metrics: passing site count, dropped site count.
- Tools: bcftools.
- Defaults: default `bcftools`; rationale lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/filter.yaml`, `domain/vcf/index.yaml`.

### vcf.gl_propagation {#vcf-gl-propagation}
- Status: supported.
- Purpose: preserve and propagate GL/PL evidence across downstream filtering and normalization boundaries.
- Inputs/Outputs: GL-bearing VCF → GL-propagated VCF.
- Metrics: retained GL field count, dropped-field warning count.
- Tools: bcftools, angsd.
- Defaults: default `bcftools`; rationale lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/gl_propagation.yaml`, `docs/20-science/vcf/ROADMAP.md`.

### vcf.stats {#vcf-stats}
- Status: supported.
- Purpose: emit required summary metrics for quality gating and downstream review.
- Inputs/Outputs: filtered VCF → stats report.
- Metrics: site totals, SNP/indel breakdown, filter summaries.
- Tools: bcftools.
- Defaults: default `bcftools`; rationale lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/stats.yaml`, `domain/vcf/index.yaml`.

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
- Every VCF stage manifest must appear here exactly once, including planned downstream stages.

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

### vcf.qc {#vcf-qc}
- Status: planned.
- Purpose: apply cohort-level QC summaries and threshold checks before downstream structure or imputation analysis.
- Inputs/Outputs: filtered or stats-enriched VCF → QC report.
- Metrics: missingness summaries, MAF guardrails, QC status.
- Tools: plink, plink2.
- Defaults: planned default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/qc.yaml`, `domain/vcf/docs/IMPUTATION_CONTRACT.md`.

### vcf.pca {#vcf-pca}
- Status: planned.
- Purpose: compute principal-component projections for population-structure interpretation.
- Inputs/Outputs: LD-pruned VCF matrix → PCA report.
- Metrics: explained variance, PC projections.
- Tools: plink2, eigensoft.
- Defaults: planned default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/pca.yaml`, `docs/20-science/vcf/POPULATION_STRUCTURE.md`.

### vcf.admixture {#vcf-admixture}
- Status: planned.
- Purpose: estimate ancestry-mixture style summaries from cohort-level variant matrices.
- Inputs/Outputs: VCF matrix → admixture report.
- Metrics: ancestry-component summaries, admixture status.
- Tools: plink, plink2.
- Defaults: planned default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/admixture.yaml`, `docs/20-science/vcf/POPULATION_STRUCTURE.md`.

### vcf.population_structure {#vcf-population-structure}
- Status: planned.
- Purpose: emit higher-level structure summaries that combine PCA- or clustering-oriented evidence.
- Inputs/Outputs: filtered cohort VCF → population-structure report.
- Metrics: PCA variance, cluster assignment summaries.
- Tools: plink, plink2, eigensoft.
- Defaults: planned default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/population_structure.yaml`, `docs/20-science/vcf/POPULATION_STRUCTURE.md`.

### vcf.phasing {#vcf-phasing}
- Status: supported.
- Purpose: phase cohort haplotypes before downstream imputation or IBD inference.
- Inputs/Outputs: filtered VCF plus panel metadata → phased VCF, phasing QC, and phase-set-aware metrics.
- Metrics: phased genotype count, unphased genotype count, phase-set count, switch-error-compatible status.
- Tools: beagle, shapeit5, eagle.
- Defaults: supported default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/phasing.yaml`, `docs/20-science/vcf/IMPUTATION_SCOPE.md`.

### vcf.prepare_reference_panel {#vcf-prepare-reference-panel}
- Status: planned.
- Purpose: normalize and prepare reference panels before phasing or imputation entry.
- Inputs/Outputs: raw panel VCF/BCF → prepared panel report.
- Metrics: panel normalization status, prepared variant count.
- Tools: bcftools.
- Defaults: planned default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/prepare_reference_panel.yaml`, `docs/20-science/vcf/IMPUTATION_SCOPE.md`.

### vcf.imputation_metrics {#vcf-imputation-metrics}
- Status: planned.
- Purpose: describe the multi-tool imputation family admitted for downstream panel-based inference.
- Inputs/Outputs: phased VCF plus panel metadata → imputation report.
- Metrics: imputation status, imputed variant count.
- Tools: beagle, glimpse, impute5, minimac4.
- Defaults: planned default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/imputation.yaml`, `docs/20-science/vcf/IMPUTATION_METHODS.md`.

### vcf.impute {#vcf-impute}
- Status: planned.
- Purpose: execute explicit imputation with a pinned panel-backed backend.
- Inputs/Outputs: phased VCF plus panel metadata → imputed VCF and report.
- Metrics: imputed site count, retained posterior or dosage summaries.
- Tools: beagle, glimpse, impute5, minimac4.
- Defaults: planned default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/impute.yaml`, `docs/20-science/vcf/IMPUTATION_METHODS.md`.

### vcf.postprocess {#vcf-postprocess}
- Status: planned.
- Purpose: normalize INFO, FILTER, and FORMAT surfaces after imputation or downstream transforms.
- Inputs/Outputs: imputed VCF → postprocessed VCF and report.
- Metrics: normalized record count, INFO/FILTER rewrite count.
- Tools: bcftools.
- Defaults: planned default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/postprocess.yaml`, `domain/vcf/docs/IMPUTATION_CONTRACT.md`.

### vcf.ibd {#vcf-ibd}
- Status: planned.
- Purpose: estimate pairwise IBD segments for relatedness and demographic downstreams.
- Inputs/Outputs: cohort VCF → IBD segments and report.
- Metrics: segment count, shared cM summaries.
- Tools: germline, ibdhap.
- Defaults: planned default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/ibd.yaml`, `docs/20-science/vcf/IBD.md`.

### vcf.roh {#vcf-roh}
- Status: planned.
- Purpose: estimate runs of homozygosity burden and segment distribution.
- Inputs/Outputs: cohort VCF → ROH report.
- Metrics: ROH count, total ROH length, length-bin summaries.
- Tools: plink2.
- Defaults: planned default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/roh.yaml`, `docs/20-science/vcf/ROH.md`.

### vcf.demography {#vcf-demography}
- Status: planned.
- Purpose: estimate recent Ne-style demography summaries from IBD-derived evidence.
- Inputs/Outputs: IBD summaries → demography report.
- Metrics: recent Ne, time-series summaries.
- Tools: ibdne.
- Defaults: planned default lives in `domain/vcf/docs/DEFAULT_SETTINGS.md`.
- References: `domain/vcf/stages/demography.yaml`, `docs/20-science/vcf/DEMOGRAPHY.md`.

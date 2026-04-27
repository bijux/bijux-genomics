# BAM Stage Catalog

## What
Canonical stage definitions for BAM pipelines.

## Why
Defines expectations for artifacts, metrics, defaults, and tool coverage.

## Non-goals
- Tool selection logic.

## Contracts
- Each stage must declare purpose, inputs/outputs, metrics, tools, defaults, references.

## Examples
- bam.markdup outputs a deduplicated BAM and duplicate metrics.

## Failure modes
- Missing metrics or outputs fail contract validation.

### bam.align {#bam-align}
- Purpose: map reads to reference.
- Inputs/Outputs: reads → align.bam/bai.
- Metrics: alignment rate, MAPQ.
- Tools: bwa, bowtie2.
- Defaults: align defaults.
- References: BWA, Bowtie2.

### bam.validate {#bam-validate}
- Status: supported.
- Purpose: validate BAM integrity.
- Inputs/Outputs: bam → validation report.
- Metrics: format/flag checks.
- Tools: samtools, bedtools, bamtools.
- Defaults: validation defaults.
- References: SAMtools plus BAM-structure validation helpers.

### bam.qc_pre {#bam-qc-pre}
- Status: planned.
- Purpose: baseline QC before filtering.
- Inputs/Outputs: bam → metrics_json.
- Metrics: pre‑QC summary.
- Tools: samtools.
- Defaults: qc defaults.
- References: SAMtools.

### bam.mapping_summary {#bam-mapping-summary}
- Status: supported.
- Purpose: emit observational mapping summaries before downstream interpretation gates.
- Inputs/Outputs: bam → mapping summary report.
- Metrics: mapped reads, alignment rate.
- Tools: samtools.
- Defaults: default `samtools`; rationale lives in `domain/bam/docs/DEFAULT_SETTINGS.md`.
- References: SAMtools.

### bam.filter {#bam-filter}
- Status: supported.
- Purpose: filter low‑quality alignments.
- Inputs/Outputs: bam → filtered.bam/bai.
- Metrics: filtered counts.
- Tools: samtools, bedtools, bamtools.
- Defaults: filter defaults.
- References: SAMtools plus interval-aware BAM filtering helpers.

### bam.mapq_filter {#bam-mapq-filter}
- Status: supported.
- Purpose: apply MAPQ-specific retention rules without conflating them with broader filtering policy.
- Inputs/Outputs: bam → MAPQ-filtered BAM and report.
- Metrics: reads retained fraction, mean MAPQ post-filter.
- Tools: samtools, bamtools.
- Defaults: default `samtools`; rationale lives in `domain/bam/docs/DEFAULT_SETTINGS.md`.
- References: SAMtools and BAMTools command surfaces.

### bam.length_filter {#bam-length-filter}
- Status: supported.
- Purpose: gate retained alignments by minimum fragment or read length.
- Inputs/Outputs: bam → length-filtered BAM and report.
- Metrics: reads retained fraction.
- Tools: samtools, picard.
- Defaults: default `samtools`; rationale lives in `domain/bam/docs/DEFAULT_SETTINGS.md`.
- References: SAMtools and Picard QC/reporting surfaces.

### bam.markdup {#bam-markdup}
- Status: planned.
- Purpose: mark duplicates.
- Inputs/Outputs: bam → markdup.bam/bai.
- Metrics: duplicate rate.
- Tools: picard, samtools.
- Defaults: markdup defaults.
- References: Picard and SAMtools duplicate-marking surfaces.

### bam.duplication_metrics {#bam-duplication-metrics}
- Status: supported.
- Purpose: report duplicate burden without requiring the mutation-oriented markdup stage to be promoted.
- Inputs/Outputs: bam → duplication metrics report.
- Metrics: duplication rate, duplicate histogram area.
- Tools: samtools, picard.
- Defaults: default `samtools`; rationale lives in `domain/bam/docs/DEFAULT_SETTINGS.md`.
- References: SAMtools and Picard duplication metrics.

### bam.complexity {#bam-complexity}
- Status: planned.
- Purpose: estimate library complexity.
- Inputs/Outputs: bam → complexity metrics.
- Metrics: complexity curves.
- Tools: preseq.
- Defaults: complexity defaults.
- References: preseq.

### bam.coverage {#bam-coverage}
- Status: supported.
- Purpose: coverage summaries.
- Inputs/Outputs: bam → coverage report.
- Metrics: depth/breadth.
- Tools: mosdepth, samtools.
- Defaults: coverage defaults.
- References: mosdepth and SAMtools depth summaries.

### bam.endogenous_content {#bam-endogenous-content}
- Status: supported.
- Purpose: estimate endogenous-content ratio from governed mapping summaries.
- Inputs/Outputs: bam → endogenous-content report.
- Metrics: endogenous content ratio.
- Tools: samtools.
- Defaults: default `samtools`; rationale lives in `domain/bam/docs/DEFAULT_SETTINGS.md`.
- References: SAMtools mapping summaries and endogenous-content governance.

### bam.damage {#bam-damage}
- Status: supported.
- Purpose: aDNA damage profiling.
- Inputs/Outputs: bam → damage metrics.
- Metrics: misincorporation patterns.
- Tools: mapdamage2, pydamage, damageprofiler, ngsbriggs, addeam, pmdtools.
- Defaults: damage defaults.
- References: mapDamage2, pyDamage, DamageProfiler, ngsBriggs, AdDeam, PMDtools.

### bam.authenticity {#bam-authenticity}
- Status: supported.
- Purpose: authenticity estimation.
- Inputs/Outputs: bam → authenticity metrics.
- Metrics: cytosine deamination/authenticity.
- Tools: authenticct, pmdtools, damageprofiler.
- Defaults: authenticity defaults.
- References: AuthentiCT, PMDtools, DamageProfiler.

### bam.contamination {#bam-contamination}
- Status: supported.
- Purpose: contamination estimation.
- Inputs/Outputs: bam → contamination metrics.
- Metrics: contamination rates.
- Tools: schmutzi, verifybamid2, contammix.
- Defaults: contamination defaults.
- References: Schmutzi, VerifyBamID2, ContamMix.

### bam.sex {#bam-sex}
- Status: supported.
- Purpose: sex inference.
- Inputs/Outputs: bam → sex metrics.
- Metrics: sex inference stats.
- Tools: rxy, yleaf, angsd.
- Defaults: sex defaults.
- References: RXY, Yleaf, ANGSD.

### bam.bias_mitigation {#bam-bias-mitigation}
- Purpose: mitigate GC/length bias.
- Inputs/Outputs: bam → bias metrics.
- Metrics: bias measures.
- Tools: in‑house.
- Defaults: bias defaults.
- References: internal.

### bam.recalibration {#bam-recalibration}
- Purpose: BQSR.
- Inputs/Outputs: bam → recal.bam/bai.
- Metrics: recalibration stats.
- Tools: GATK.
- Defaults: recal defaults.
- References: GATK.

### bam.haplogroups {#bam-haplogroups}
- Purpose: haplogroup inference.
- Inputs/Outputs: bam → haplogroup metrics.
- Metrics: haplogroup calls.
- Tools: Yleaf.
- Defaults: haplogroup defaults.
- References: Yleaf.

### bam.genotyping {#bam-genotyping}
- Purpose: genotype summary.
- Inputs/Outputs: bam → genotyping metrics.
- Metrics: variant summary.
- Tools: toolchain‑specific.
- Defaults: genotyping defaults.
- References: toolchain docs.

### bam.kinship {#bam-kinship}
- Status: supported.
- Purpose: relatedness inference.
- Inputs/Outputs: bam → kinship metrics.
- Metrics: kinship coefficients.
- Tools: king, angsd.
- Defaults: kinship defaults.
- References: KING, ANGSD.

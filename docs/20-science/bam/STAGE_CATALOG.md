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
- Purpose: validate BAM integrity.
- Inputs/Outputs: bam → validation report.
- Metrics: format/flag checks.
- Tools: samtools.
- Defaults: validation defaults.
- References: SAMtools.

### bam.qc_pre {#bam-qc-pre}
- Purpose: baseline QC before filtering.
- Inputs/Outputs: bam → metrics_json.
- Metrics: pre‑QC summary.
- Tools: samtools/other QC tools.
- Defaults: qc defaults.
- References: SAMtools.

### bam.filter {#bam-filter}
- Purpose: filter low‑quality alignments.
- Inputs/Outputs: bam → filtered.bam/bai.
- Metrics: filtered counts.
- Tools: samtools.
- Defaults: filter defaults.
- References: SAMtools.

### bam.markdup {#bam-markdup}
- Purpose: mark duplicates.
- Inputs/Outputs: bam → markdup.bam/bai.
- Metrics: duplicate rate.
- Tools: picard.
- Defaults: markdup defaults.
- References: Picard.

### bam.complexity {#bam-complexity}
- Purpose: estimate library complexity.
- Inputs/Outputs: bam → complexity metrics.
- Metrics: complexity curves.
- Tools: preseq.
- Defaults: complexity defaults.
- References: preseq.

### bam.coverage {#bam-coverage}
- Purpose: coverage summaries.
- Inputs/Outputs: bam → coverage report.
- Metrics: depth/breadth.
- Tools: mosdepth.
- Defaults: coverage defaults.
- References: mosdepth.

### bam.damage {#bam-damage}
- Purpose: aDNA damage profiling.
- Inputs/Outputs: bam → damage metrics.
- Metrics: misincorporation patterns.
- Tools: mapDamage2, pyDamage.
- Defaults: damage defaults.
- References: mapDamage2, pyDamage.

### bam.authenticity {#bam-authenticity}
- Purpose: authenticity estimation.
- Inputs/Outputs: bam → authenticity metrics.
- Metrics: cytosine deamination/authenticity.
- Tools: authenticCT.
- Defaults: authenticity defaults.
- References: authenticCT.

### bam.contamination {#bam-contamination}
- Purpose: contamination estimation.
- Inputs/Outputs: bam → contamination metrics.
- Metrics: contamination rates.
- Tools: ANGSD.
- Defaults: contamination defaults.
- References: ANGSD.

### bam.sex {#bam-sex}
- Purpose: sex inference.
- Inputs/Outputs: bam → sex metrics.
- Metrics: sex inference stats.
- Tools: RXY.
- Defaults: sex defaults.
- References: RXY.

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
- Purpose: relatedness inference.
- Inputs/Outputs: bam → kinship metrics.
- Metrics: kinship coefficients.
- Tools: KING.
- Defaults: kinship defaults.
- References: KING.

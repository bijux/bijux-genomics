# BAM Stage Catalog

## What
Canonical stage definitions for BAM pipelines.

## Why
Defines expectations for artifacts, metrics, defaults, and tool coverage.

## Non-goals
- Replacing the tool-selection surface in [TOOLS_ROSTER.md](TOOLS_ROSTER.md).

## Contracts
- The governed BAM stage inventory lives in [../../../domain/bam/index.yaml](../../../domain/bam/index.yaml).
- Default-tool rationale lives in
  [../../../domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- Stage/tool status and admission surfaces stay aligned with [TOOLS_ROSTER.md](TOOLS_ROSTER.md).

## Examples
- bam.markdup outputs a deduplicated BAM and duplicate metrics.

## Failure modes
- Missing metrics or outputs fail contract validation.

### bam.align {#bam-align}
- Status: supported.
- Purpose: emit governed BAM alignment reports from already materialized BAM-domain inputs.
- Inputs/Outputs: bam → align report.
- Metrics: alignment rate, MAPQ.
- Tools: bwa, bowtie2.
- Defaults: default `bwa`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: BWA and Bowtie2 aligner contracts.

### bam.validate {#bam-validate}
- Status: supported.
- Purpose: validate BAM integrity.
- Inputs/Outputs: bam → validation report.
- Metrics: format/flag checks.
- Tools: samtools, bedtools, bamtools.
- Defaults: default `samtools`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: SAMtools plus BAM-structure validation helpers.

### bam.qc_pre {#bam-qc-pre}
- Status: planned.
- Purpose: baseline QC before filtering.
- Inputs/Outputs: bam → pre-QC report.
- Metrics: pre‑QC summary.
- Tools: samtools.
- Defaults: default `samtools`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: SAMtools.

### bam.mapping_summary {#bam-mapping-summary}
- Status: supported.
- Purpose: emit observational mapping summaries before downstream interpretation gates.
- Inputs/Outputs: bam → mapping summary report.
- Metrics: mapped reads, alignment rate.
- Tools: samtools.
- Defaults: default `samtools`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: SAMtools.

### bam.filter {#bam-filter}
- Status: supported.
- Purpose: filter low‑quality alignments.
- Inputs/Outputs: bam → filtered BAM and report.
- Metrics: filtered counts.
- Tools: samtools, bedtools, bamtools.
- Defaults: default `samtools`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: SAMtools plus interval-aware BAM filtering helpers.

### bam.mapq_filter {#bam-mapq-filter}
- Status: supported.
- Purpose: apply MAPQ-specific retention rules without conflating them with broader filtering policy.
- Inputs/Outputs: bam → MAPQ-filtered BAM and report.
- Metrics: reads retained fraction, mean MAPQ post-filter.
- Tools: samtools, bamtools.
- Defaults: default `samtools`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: SAMtools and BAMTools command surfaces.

### bam.length_filter {#bam-length-filter}
- Status: supported.
- Purpose: gate retained alignments by minimum fragment or read length.
- Inputs/Outputs: bam → length-filtered BAM and report.
- Metrics: reads retained fraction.
- Tools: samtools, picard.
- Defaults: default `samtools`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: SAMtools and Picard QC/reporting surfaces.

### bam.markdup {#bam-markdup}
- Status: planned.
- Purpose: mark duplicates.
- Inputs/Outputs: bam → duplicate-marking report.
- Metrics: duplicate rate.
- Tools: picard, samtools.
- Defaults: default `samtools`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: Picard and SAMtools duplicate-marking surfaces.

### bam.duplication_metrics {#bam-duplication-metrics}
- Status: supported.
- Purpose: report duplicate burden without requiring the mutation-oriented markdup stage to be promoted.
- Inputs/Outputs: bam → duplication metrics report.
- Metrics: duplication rate, duplicate histogram area.
- Tools: samtools, picard.
- Defaults: default `samtools`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: SAMtools and Picard duplication metrics.

### bam.complexity {#bam-complexity}
- Status: planned.
- Purpose: estimate library complexity.
- Inputs/Outputs: bam → complexity report.
- Metrics: complexity curves.
- Tools: preseq.
- Defaults: default `preseq`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: preseq.

### bam.coverage {#bam-coverage}
- Status: supported.
- Purpose: coverage summaries.
- Inputs/Outputs: bam → coverage report.
- Metrics: depth/breadth.
- Tools: mosdepth, samtools, bedtools.
- Defaults: default `mosdepth`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: mosdepth, SAMtools depth summaries, and BEDTools interval coverage.

### bam.endogenous_content {#bam-endogenous-content}
- Status: supported.
- Purpose: estimate endogenous-content ratio from governed mapping summaries.
- Inputs/Outputs: bam → endogenous-content report.
- Metrics: endogenous content ratio.
- Tools: samtools.
- Defaults: default `samtools`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: SAMtools mapping summaries and endogenous-content governance.

### bam.damage {#bam-damage}
- Status: supported.
- Purpose: aDNA damage profiling.
- Inputs/Outputs: bam → damage metrics.
- Metrics: misincorporation patterns.
- Tools: mapdamage2, pydamage, damageprofiler, ngsbriggs, addeam, pmdtools.
- Defaults: default `mapdamage2`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: mapDamage2, pyDamage, DamageProfiler, ngsBriggs, AdDeam, PMDtools.

### bam.authenticity {#bam-authenticity}
- Status: supported.
- Purpose: authenticity estimation.
- Inputs/Outputs: bam → authenticity metrics.
- Metrics: cytosine deamination/authenticity.
- Tools: authenticct, pmdtools, damageprofiler.
- Defaults: default `authenticct`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: AuthentiCT, PMDtools, DamageProfiler.

### bam.contamination {#bam-contamination}
- Status: supported.
- Purpose: contamination estimation.
- Inputs/Outputs: bam → contamination metrics.
- Metrics: contamination rates.
- Tools: schmutzi, verifybamid2, contammix.
- Defaults: default `schmutzi`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: Schmutzi, VerifyBamID2, ContamMix.

### bam.sex {#bam-sex}
- Status: supported.
- Purpose: sex inference.
- Inputs/Outputs: bam → sex metrics.
- Metrics: sex inference stats.
- Tools: rxy, yleaf, angsd.
- Defaults: default `rxy`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: RXY, Yleaf, ANGSD.

### bam.bias_mitigation {#bam-bias-mitigation}
- Status: planned.
- Purpose: record bias-mitigation decisions that should stay separate from pure bias observation.
- Inputs/Outputs: bam → bias-mitigation report.
- Metrics: bias-mitigation summary, operator notes.
- Tools: samtools.
- Defaults: default `samtools`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: SAMtools plus governed bias-mitigation policy notes.

### bam.recalibration {#bam-recalibration}
- Status: planned.
- Purpose: record recalibration decisions and reportability boundaries before BQSR promotion.
- Inputs/Outputs: bam → recalibration report.
- Metrics: recalibration stats.
- Tools: gatk.
- Defaults: default `gatk`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: GATK.

### bam.haplogroups {#bam-haplogroups}
- Status: supported.
- Purpose: infer haplogroups from governed BAM-aligned Y-panel evidence with explicit readiness and contamination guardrails.
- Inputs/Outputs: bam → haplogroups report.
- Metrics: haplogroup calls.
- Tools: yleaf.
- Defaults: default `yleaf`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: Yleaf.

### bam.genotyping {#bam-genotyping}
- Status: supported.
- Purpose: summarize low-depth genotyping from BAM evidence with owned candidate-sites and target-regions contracts.
- Inputs/Outputs: bam → genotyping report.
- Metrics: variant summary.
- Tools: angsd.
- Defaults: default `angsd`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: ANGSD genotype-likelihood/reporting surfaces.

### bam.kinship {#bam-kinship}
- Status: supported.
- Purpose: relatedness inference.
- Inputs/Outputs: bam → kinship metrics.
- Metrics: kinship coefficients.
- Tools: king, angsd.
- Defaults: default `king`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: KING, ANGSD.

### bam.insert_size {#bam-insert-size}
- Status: planned.
- Purpose: summarize insert-size distributions when paired-end metadata is available.
- Inputs/Outputs: bam → insert-size report.
- Metrics: insert-size mean, insert-size standard deviation.
- Tools: picard.
- Defaults: default `picard`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: Picard insert-size metrics.

### bam.gc_bias {#bam-gc-bias}
- Status: planned.
- Purpose: report GC/AT dropout and other alignment-linked GC-bias summaries.
- Inputs/Outputs: bam → GC-bias report.
- Metrics: GC dropout, AT dropout.
- Tools: picard.
- Defaults: default `picard`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: Picard GC-bias metrics.

### bam.overlap_correction {#bam-overlap-correction}
- Status: planned.
- Purpose: correct overlapping paired-read evidence when clip/consensus rules are explicitly requested.
- Inputs/Outputs: bam → overlap-correction report.
- Metrics: overlap-corrected read pairs.
- Tools: bamutil.
- Defaults: default `bamutil`; rationale lives in [domain/bam/docs/DEFAULT_SETTINGS.md](../../../domain/bam/docs/DEFAULT_SETTINGS.md).
- References: bamUtil overlap-clipping/correction surfaces.

# FASTQ Stage Catalog

## What
Canonical stage definitions for the FASTQ pipeline.

## Why
Defines expectations for artifacts, metrics, defaults, and tool coverage.

## Non-goals
- Tool selection logic.

## Contracts
- Each stage must declare purpose, inputs/outputs, metrics, tools, defaults, references.

## Examples
- fastq.trim uses trimming tools and outputs trimmed reads.

## Failure modes
- Missing metrics or outputs fail contract validation.

### fastq.validate_pre {#fastq-validate-pre}
- Purpose: validate FASTQ format and counts.
- Inputs/Outputs: reads → validation_report.
- Metrics: read counts, format errors.
- Tools: fastqvalidator, seqkit.
- Defaults: pipeline defaults for validation thresholds.
- References: FASTQ‑Validator, SeqKit.

### fastq.detect_adapters {#fastq-detect-adapters}
- Purpose: detect adapter contamination.
- Inputs/Outputs: reads → adapter summary.
- Metrics: adapter composition, read loss.
- Tools: fastp.
- Defaults: pipeline defaults for adapter detection.
- References: fastp.

### fastq.trim {#fastq-trim}
- Purpose: trim adapters/low‑quality bases.
- Inputs/Outputs: reads → trimmed_reads.
- Metrics: trimming counts, retention.
- Tools: fastp, cutadapt, trimmomatic.
- Defaults: trimming thresholds from pipeline defaults.
- References: fastp, Cutadapt, Trimmomatic.

### fastq.filter {#fastq-filter}
- Purpose: remove low‑quality reads.
- Inputs/Outputs: reads → filtered_reads.
- Metrics: read loss reasons, retention.
- Tools: seqkit, prinseq, fastp.
- Defaults: filter thresholds from pipeline defaults.
- References: SeqKit, PRINSEQ.

### fastq.stats_neutral {#fastq-stats-neutral}
- Purpose: compute baseline read stats.
- Inputs/Outputs: reads → metrics_json.
- Metrics: length/quality summary.
- Tools: seqkit_stats.
- Defaults: stats defaults.
- References: SeqKit.

### fastq.merge {#fastq-merge}
- Purpose: merge paired‑end reads.
- Inputs/Outputs: paired reads → merged_reads.
- Metrics: merge rate, overlap stats.
- Tools: pear, flash2, bbmerge, vsearch.
- Defaults: merge defaults.
- References: PEAR, FLASH2, VSEARCH.

### fastq.correct {#fastq-correct}
- Purpose: correct sequencing errors.
- Inputs/Outputs: reads → corrected_reads.
- Metrics: correction rates.
- Tools: rcorrector, spades/bayeshammer, lighter, musket.
- Defaults: correction defaults.
- References: RCorrector, SPAdes.

### fastq.umi {#fastq-umi}
- Purpose: handle UMI‑tagged reads.
- Inputs/Outputs: reads → umi_reads.
- Metrics: UMI grouping, consensus stats.
- Tools: umi_tools.
- Defaults: umi defaults.
- References: UMI‑tools.

### fastq.qc_post {#fastq-qc-post}
- Purpose: post‑processing QC aggregation.
- Inputs/Outputs: reads → report_html.
- Metrics: QC summary artifacts.
- Tools: multiqc.
- Defaults: qc defaults.
- References: MultiQC.

### fastq.screen {#fastq-screen}
- Purpose: contamination screening.
- Inputs/Outputs: reads → screen report.
- Metrics: classification summary.
- Tools: kraken2, centrifuge.
- Defaults: screening defaults.
- References: Kraken2, Centrifuge.

### fastq.preprocess {#fastq-preprocess}
- Purpose: composite pipeline stage.
- Inputs/Outputs: pipeline outputs.
- Metrics: pipeline‑level metrics.
- Tools: planner.
- Defaults: pipeline defaults.
- References: internal.

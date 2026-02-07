# Tool Index

## What
Alphabetical index of tools used in FASTQ and BAM stages.

## Why
Provides a single lookup for tool coverage.

## Non-goals
- Full tool manuals.

## Contracts
- Tool IDs are canonical and referenced by stage catalogs.

## Examples
- fastp → FASTQ trim stage.

## Failure modes
- Missing tool references in stage catalogs.

## Index
- ANGSD → [bam.contamination](bam/STAGE_CATALOG.md#bam-contamination)
- BBMerge → [fastq.merge](fastq/STAGE_CATALOG.md#fastq-merge)
- Bowtie2 → [bam.align](bam/STAGE_CATALOG.md#bam-align)
- BWA → [bam.align](bam/STAGE_CATALOG.md#bam-align)
- Cutadapt → [fastq.trim](fastq/STAGE_CATALOG.md#fastq-trim)
- fastp → [fastq.detect_adapters](fastq/STAGE_CATALOG.md#fastq-detect-adapters), [fastq.trim](fastq/STAGE_CATALOG.md#fastq-trim)
- FASTQ‑Validator → [fastq.validate_pre](fastq/STAGE_CATALOG.md#fastq-validate-pre)
- FLASH2 → [fastq.merge](fastq/STAGE_CATALOG.md#fastq-merge)
- GATK → [bam.recalibration](bam/STAGE_CATALOG.md#bam-recalibration)
- KING → [bam.kinship](bam/STAGE_CATALOG.md#bam-kinship)
- Kraken2 → [fastq.screen](fastq/STAGE_CATALOG.md#fastq-screen)
- mapDamage2 → [bam.damage](bam/STAGE_CATALOG.md#bam-damage)
- mosdepth → [bam.coverage](bam/STAGE_CATALOG.md#bam-coverage)
- MultiQC → [fastq.qc_post](fastq/STAGE_CATALOG.md#fastq-qc-post)
- PEAR → [fastq.merge](fastq/STAGE_CATALOG.md#fastq-merge)
- Picard → [bam.markdup](bam/STAGE_CATALOG.md#bam-markdup)
- preseq → [bam.complexity](bam/STAGE_CATALOG.md#bam-complexity)
- RCorrector → [fastq.correct](fastq/STAGE_CATALOG.md#fastq-correct)
- RXY → [bam.sex](bam/STAGE_CATALOG.md#bam-sex)
- SAMtools → [bam.validate](bam/STAGE_CATALOG.md#bam-validate)
- SeqKit → [fastq.filter](fastq/STAGE_CATALOG.md#fastq-filter)
- UMI‑tools → [fastq.umi](fastq/STAGE_CATALOG.md#fastq-umi)
- VSEARCH → [fastq.merge](fastq/STAGE_CATALOG.md#fastq-merge)
- Yleaf → [bam.haplogroups](bam/STAGE_CATALOG.md#bam-haplogroups)

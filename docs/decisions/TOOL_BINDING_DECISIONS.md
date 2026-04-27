# Tool Binding Decisions

This record captures tools whose binding shape spans multiple domains or differs between
`stage_ids` and `bindings`, and documents why the binding was moved/expanded.

- bowtie2:
  - decision: keep canonical role as `aligner` and allow use in FASTQ host depletion and BAM alignment.
  - reason: host depletion in FASTQ is alignment-driven; contaminant screening remains screen-tool based.
  - affected bindings: `bam.align`, `fastq.deplete_host`.
  - date: 2026-02-11.
- angsd:
  - decision: keep binding in BAM authenticity/contamination analysis only.
  - reason: estimator semantics are specific to BAM-level damage/authenticity workflows.
- adapterremoval:
  - decision: keep explicit dual binding across FASTQ trimming, terminal-damage pretrim, and pair merging.
  - reason: the backend legitimately spans adapter-aware trimming and overlap merge semantics in aDNA-oriented FASTQ preparation.
  - affected bindings: `fastq.trim_reads`, `fastq.trim_terminal_damage`, `fastq.merge_pairs`.
  - date: 2026-04-27.
- bamtools:
  - decision: keep BAM-only binding for BAM transform/metrics stages.
  - reason: utility is BAM-structural and not FASTQ-domain compatible.
- bbduk:
  - decision: allow FASTQ trim/filter roles through explicit stage bindings.
  - reason: same tool serves distinct semantics by stage contract.
- bedtools:
  - decision: keep BAM/coverage analytics binding only.
  - reason: interval operations are downstream of alignment outputs.
- cutadapt:
  - decision: keep explicit FASTQ bindings for read trimming, primer normalization, and terminal-damage pretrim.
  - reason: one backend spans several governed sequence-editing contracts, but each stage preserves a distinct intent and artifact surface.
  - affected bindings: `fastq.trim_reads`, `fastq.normalize_primers`, `fastq.trim_terminal_damage`.
  - date: 2026-04-27.
- fastp:
  - decision: keep FASTQ trim/filter/QC bindings and disallow BAM roles.
  - reason: algorithm and outputs are read-level preprocessing semantics.
- fastq_scan:
  - decision: keep explicit bindings in FASTQ validation and overrepresented-sequence profiling.
  - reason: the scanner is diagnostic-only, but it serves both structural validation and sequence-content reporting contracts.
  - affected bindings: `fastq.validate_reads`, `fastq.profile_overrepresented_sequences`.
  - date: 2026-04-27.
- fastqc:
  - decision: keep explicit bindings in FASTQ validation, adapter detection, and overrepresented-sequence profiling.
  - reason: FastQC is report-oriented and spans multiple FASTQ diagnostic surfaces without mutating reads.
  - affected bindings: `fastq.validate_reads`, `fastq.detect_adapters`, `fastq.profile_overrepresented_sequences`.
  - date: 2026-04-27.
- leehom:
  - decision: keep explicit FASTQ bindings for pair merging and trim-stage overlap handling.
  - reason: ancient-DNA overlap merge and adapter cleanup are coupled in this backend and need stage-specific governed outputs.
  - affected bindings: `fastq.trim_reads`, `fastq.merge_pairs`.
  - date: 2026-04-27.
- pmdtools:
  - decision: keep BAM authenticity/damage role binding.
  - reason: PMD signal interpretation is BAM-domain specific.
- prinseq:
  - decision: keep explicit FASTQ bindings for trimming, general filtering, and low-complexity filtering.
  - reason: PRINSEQ exposes distinct governed filtering contracts through one backend family.
  - affected bindings: `fastq.trim_reads`, `fastq.filter_reads`, `fastq.filter_low_complexity`.
  - date: 2026-04-27.
- samtools:
  - decision: use explicit multi-binding by stage role (prepare_reference, qc, metrics, transform).
  - reason: one binary legitimately spans multiple BAM/FASTQ support stages.
- seqkit:
  - decision: keep explicit FASTQ bindings for trimming, filtering, abundance normalization, sequence profiling, and terminal-damage pretrim.
  - reason: different `seqkit` subcommands satisfy distinct governed contracts, so binding shape must stay stage-specific.
  - affected bindings: `fastq.trim_reads`, `fastq.filter_reads`, `fastq.normalize_abundance`, `fastq.trim_terminal_damage`, `fastq.profile_overrepresented_sequences`.
  - date: 2026-04-27.
- seqkit_stats:
  - decision: keep explicit FASTQ bindings for profile summaries and read-length summaries only.
  - reason: the stats surface is analytic and should remain limited to report-only FASTQ profiling stages.
  - affected bindings: `fastq.profile_reads`, `fastq.profile_read_lengths`.
  - date: 2026-04-27.
- vsearch:
  - decision: keep explicit FASTQ bindings for pair merging, chimera removal, and OTU clustering.
  - reason: one backend spans multiple amplicon/paired-end transforms that must remain separated by stage contract.
  - affected bindings: `fastq.merge_pairs`, `fastq.remove_chimeras`, `fastq.cluster_otus`.
  - date: 2026-04-27.

## Purpose
This document defines the intended behavior and navigation contract for this topic.

## Scope
Applies only to the files and workflows referenced in this document.

## Non-goals
- Not a replacement for lower-level implementation or crate-specific contracts.

## Contracts
- Content here is normative where explicitly stated.

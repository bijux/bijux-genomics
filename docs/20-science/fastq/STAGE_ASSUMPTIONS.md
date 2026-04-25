# FASTQ Stage Scientific Assumptions

This document maps FASTQ stage-level scientific assumptions used in the pre-HPC scope.
Source of truth remains `domain/fastq/stages/*.yaml` (`assumptions` field).

## Stage assumptions
- `fastq.index_reference`: reference sequence is representative and build inputs are stable.
- `fastq.validate_reads`: FASTQ records are expected to be well-formed after ingest.
- `fastq.detect_adapters`: adapter bank captures dominant library prep adapters.
- `fastq.trim_reads`: adapter/quality trimming improves downstream signal-to-noise.
- `fastq.filter_reads`: filtering thresholds remove low-information reads without biasing core signal.
- `fastq.filter_low_complexity`: complexity thresholds remove low-information reads without collapsing legitimate repetitive biology.
- `fastq.merge_pairs`: overlapping pairs represent the same original molecule when merged.
- `fastq.deplete_host`: host reference choice and mapper sensitivity determine both privacy protection and microbial over-removal risk.
- `fastq.deplete_reference_contaminants`: decoy references represent real technical contaminants, not target signal.
- `fastq.profile_reads`: summary statistics are diagnostic, not inferential.
- `fastq.profile_read_lengths`: length summaries remain neutral only when upstream stages are unchanged.
- `fastq.profile_overrepresented_sequences`: overrepresented-sequence flags require interpretation in context of adapters, primers, and contaminants.
- `fastq.report_qc`: QC aggregates are interpretable only in context of upstream parameters.
- `fastq.screen_taxonomy`: taxonomy/classification metrics depend on database coverage/composition.
- `fastq.deplete_rrna`: rRNA database is appropriate for the studied material.
- `fastq.correct_errors`: error correction model assumptions match observed read error profile.
- `fastq.extract_umis`: UMI schema/pattern reflects library design; inline extraction must preserve read pairing and run before trimming or filtering can remove barcode-bearing sequence.
- `fastq.trim_terminal_damage`: terminal damage trimming is specific to aDNA-like libraries and must be profile- or user-selected, not a generic unknown-assay requirement.

## Contract note
Assumptions are validated for presence by domain validation; semantic interpretation remains operator responsibility.

## Purpose
This document defines the intended behavior and navigation contract for this topic.

## Scope
Applies only to the files and workflows referenced in this document.

## Non-goals
- Not a replacement for lower-level implementation or crate-specific contracts.

## Contracts
- Content here is normative where explicitly stated.

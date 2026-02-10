# FASTQ Stage Scientific Assumptions

This document maps FASTQ stage-level scientific assumptions used in the pre-HPC scope.
Source of truth remains `domain/fastq/stages/*.yaml` (`assumptions` field).

## Stage assumptions
- `fastq.prepare_reference`: reference sequence is representative and build inputs are stable.
- `fastq.validate_pre`: FASTQ records are expected to be well-formed after ingest.
- `fastq.detect_adapters`: adapter bank captures dominant library prep adapters.
- `fastq.trim`: adapter/quality trimming improves downstream signal-to-noise.
- `fastq.filter`: filtering thresholds remove low-information reads without biasing core signal.
- `fastq.merge`: overlapping pairs represent the same original molecule when merged.
- `fastq.stats_neutral`: summary statistics are diagnostic, not inferential.
- `fastq.qc_post`: QC aggregates are interpretable only in context of upstream parameters.
- `fastq.screen`: taxonomy/classification metrics depend on database coverage/composition.
- `fastq.rrna`: rRNA database is appropriate for the studied material.
- `fastq.correct`: error correction model assumptions match observed read error profile.
- `fastq.umi`: UMI schema/pattern reflects library design.

## Contract note
Assumptions are validated for presence by domain validation; semantic interpretation remains operator responsibility.

# FASTQ Domain Contract

This document defines the non-negotiable contract for the FASTQ domain in Bijux.

## Stage Meanings

- validate: structural correctness checks; never mutates reads.
- trim: adapter/quality trimming; mutates reads and may reduce read count.
- filter: removes low-quality reads; mutates reads and may reduce read count.
- stats: observational summaries; never mutates reads.
- merge: paired-end merge; mutates reads and may reduce read count.
- correct: error correction; mutates reads but should preserve read count.
- umi: UMI-aware processing; mutates reads and may reduce read count.
- qc_post: diagnostics after processing; never mutates reads.
- screen: contamination screening; never mutates reads.
- preprocess: validate -> trim -> filter -> stats pipeline.

## Invariants

- No silent read duplication.
- Pairing is preserved unless a stage explicitly allows it to break.
- Stage output is normalized to canonical names.
- Metrics must conform to schema and version.
- Header inspection detects pairing mismatches and read-name drift.

## Metrics

- All stage transitions emit a FastqDelta.
- Stages emit raw numbers only; deltas and semantic interpretations live in metrics modules.
- Metric schemas are versioned and enforced.

## Mutability Rules

Each stage declares:
- mutates_fastq
- report_only
- may_change_read_count

These annotations are required for DAG safety and are validated via manifest schema.

## What Must Never Change

- Stage IDs and their semantic meanings.
- Invariants listed above.
- Metric schema versioning discipline.
- Output normalization behavior.

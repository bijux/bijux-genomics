# FASTQ Domain Specification

This is the single authoritative FASTQ spec for Bijux. It defines pipeline stages, invariants, metrics, failure taxonomy, and optional branches. It supersedes all prior FASTQ markdown.

See also (supporting, non-authoritative):

- docs/domains/fastq_lifecycle.md
- docs/contracts/fastq_v1.md
- docs/domains/fastq_iteration1.md

## Pipeline Stages

Stage tiers are fixed as of FASTQ v1:

- Core: validate_pre, trim, merge, correct, filter, stats_neutral
- Optional: qc_post, umi, screen
- Meta: preprocess

Core stages (canonical order):

1) validate_pre
2) trim
3) filter
4) stats_neutral

Optional branches:

- merge (paired-end only)
- correct (paired-end only)
- umi (paired-end only)
- qc_post (reporting/diagnostics)
- screen (reporting/diagnostics)
- preprocess (composed pipeline)

Stage intent (summary):

- validate_pre: structural correctness and read integrity
- trim: adapter/quality trimming with canonical output normalization
- filter: read-level filtering with retention accounting
- stats_neutral: tool-agnostic read/base/length/quality summaries
- merge: paired-end merge into single-end reads
- correct: error correction on paired-end reads
- umi: UMI-aware processing of paired-end reads
- qc_post/screen: diagnostic reports; do not mutate reads
- preprocess: validate_pre -> trim -> filter -> stats_neutral orchestration

## Invariants

These must hold for all FASTQ tools and stages:

- Reads are never duplicated silently.
- Pairing is preserved unless the stage contract allows it to break.
- Output is normalized to canonical names at stage boundaries.
- Metrics must pass schema validation.
- Header inspection detects pairing mismatches and read name drift (warn by default, fail in strict mode).
- Stats is the canonical normalization boundary for read/base/length/quality summaries.

## Metrics

Metrics are semantic, not raw bags. Each stage emits:

- IntegrityMetrics: format validity, gzip integrity, pairing integrity.
- RetentionMetrics: reads/bases retained or dropped.
- QualityShiftMetrics: quality distribution deltas, length distribution shifts.
- ContaminationMetrics: adapter signal, unexpected content indicators.

All stage transitions emit a FastqDelta. No stage or tool computes deltas directly; all deltas are derived by the domain authority.

## Failure Taxonomy

Failures are classified into three buckets:

- data_error: input data is invalid or incompatible with the stage contract.
- invariants: output violates domain invariants or contract rules.
- tool_error: tool failure, non-zero exit, or unparseable output.

These classifications are stable and are used for reporting and gating.

## Optional Branch Rules

- merge requires paired-end inputs and produces merged single-end outputs.
- correct requires paired-end inputs and preserves pairing unless tool semantics explicitly allow changes.
- umi requires paired-end inputs and compatible headers.
- preprocess is composition only; it does not alter semantics beyond its stages.

## Artifact Types

Public artifact types are fixed:

- FastqSingleEnd
- FastqPairedEnd
- FastqStats

These are the only accepted inputs/outputs per stage contract.

## Contributor Contract (FASTQ)

If you add or change FASTQ tools, you must follow this contract:

- Obey the FastqStageContract for the stage.
- Use domain-provided preflight checks and header inspection.
- Use domain-provided normalization at stage boundaries.
- Do not implement ad-hoc delta logic in stage/tool code.
- Run `make test lint security` before committing.

## Authority Boundaries

- FASTQ semantics live only in `bijux_domain_fastq::core` (contracts/invariants/compatibility).
- Stages orchestrate execution and delegate semantics to the domain.
- Bench/reporting consumes domain metrics and deltas; it does not redefine semantics.

## Structural Freeze

The structural layout of `bijux-domain-fastq` is frozen as of FASTQ v1. New functionality must fit this model.

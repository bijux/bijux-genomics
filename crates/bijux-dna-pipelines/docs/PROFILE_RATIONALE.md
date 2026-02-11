# PROFILE_RATIONALE

Each pipeline profile includes tools/stages chosen for scientific intent.
Reference aDNA-specific choices in the main science docs.


## Imported Notes from PROFILE_RATIONALE_FASTQ.md

# FASTQ Profile Contract

This document defines what the FASTQ profile layer guarantees and how
`invariants_preset = "adna"` changes those guarantees.

## Base FASTQ guarantees

- Required processing stages include:
  - `fastq.validate_pre`
  - `fastq.detect_adapters`
  - `fastq.trim`
  - `fastq.filter`
  - `fastq.qc_post`
- Required parameter payloads are present and parseable for:
  - detect adapters
  - trim
  - filter

## aDNA guarantees (`invariants_preset = "adna"`)

- Includes all base FASTQ stages.
- Includes `fastq.merge` for paired-end short-fragment overlap handling.
- Uses aDNA-safe trimming defaults:
  - `trim.min_len > 0`
  - `trim.adapter_policy != "none"`
  - quality trimming enabled (`trim.q_cutoff`)
  - poly-X trimming enabled (`trim.polyx_policy`)
- Uses explicit short-read merge defaults:
  - `merge.min_len > 0`
  - `merge.merge_overlap` set
- Tool compatibility constraints:
  - `fastq.trim` tool must be one of `{adapterremoval, leehom}`
  - `fastq.merge` tool must be `leehom`

## Validation API

`validate_fastq_profile(profile)` returns a structured report:

- `valid`: boolean
- `violations`: list of invariant violations with codes and stage context

This API is used by contract tests and CLI explain surfaces to make profile
intent auditable.


## Imported Notes from PROFILE_RATIONALE_BAM.md

# BAM Profile Contract

This document defines what BAM profiles guarantee in `bijux-dna-pipelines`.

## Profile families

- `bam-to-bam__default__v1`: stable baseline BAM processing.
- `bam-to-bam__adna_shotgun__v1`: aDNA shotgun defaults.
- `bam-to-bam__adna_capture__v1`: aDNA capture defaults.

`bam_adna_profile()` is an alias of `bam_adna_shotgun_profile()`.

## Required BAM stages

All BAM profiles must include:

- `bam.validate`
- `bam.qc_pre`
- `bam.filter`
- `bam.coverage`
- `bam.damage`

## aDNA invariants

When `invariants_preset = "adna"`:

- `bam.damage` is required unless explicitly disabled with a documented justification.
- Every required BAM stage must have stage-typed defaults.
- Index-dependent BAM QC stages (`bam.coverage`, `bam.damage`, `bam.contamination`, `bam.sex`, and others) require a precondition stage: `bam.validate` or `bam.qc_pre`.

## Validation API

Use `validate_bam_profile(profile)` to get a structured `BamProfileValidationReport`:

- `valid`: overall pass/fail
- `violations`: machine-readable invariant violations with stable `code`
- `invariants_version`: profile invariant spec version


## Imported Notes from PROFILE_RATIONALE_REFERENCE_ADNA.md

# FASTQ Reference-Grade aDNA Profile Contract

Profile: `fastq-to-fastq__reference_adna__v1`

Guarantees:
- Required stages: `fastq.validate_pre`, `fastq.detect_adapters`, `fastq.trim`, `fastq.low_complexity`, `fastq.merge`, `fastq.filter`, `fastq.stats_neutral`, `fastq.qc_post`.
- aDNA trimming invariants: `trim.min_len > 0`, `trim.adapter_policy != none`, quality trimming enabled, poly-X trimming enabled.
- Pairing/library declaration: preprocess params must declare paired library mode; paired libraries require merge unless explicitly disabled.
- Optional contamination screen hook: if `fastq.screen` is enabled, `contaminant_db` must be declared.

Metrics expectations:
- `fastq.stats_neutral` includes read-length and GC distributions.
- `fastq.detect_adapters` and `fastq.qc_post` include overrepresented-sequence counts derived from FastQC data.
- `fastq.low_complexity` is used as a pre-alignment complexity/duplication proxy estimate stage.

## Scientific rigor additions

- Every profile carries an explicit `library_model` (`layout`, `udg_treatment`, `platform_hint`, `assay_kind`).
- Invariant violations include severity semantics:
  - `hard`: blocking for production validation
  - `soft`: warning-level scientific risk
- Validators can be projected into `bijux.invariants_report.v1` and emitted as `invariants_report.json`.

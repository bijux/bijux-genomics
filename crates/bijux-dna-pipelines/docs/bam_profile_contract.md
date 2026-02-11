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

# PIPELINES

Authoritative list of pipeline IDs and intended use.

Registry authority: pipeline IDs are defined in `src/registry/*` and must be unique. Uniqueness is enforced by
`tests/profiles/pipeline_ids_unique.rs` and registry snapshot tests.

## fastq-only
- `fastq-to-fastq__default__v1` — standard FASTQ preprocessing for modern data.
- `fastq-to-fastq__minimal__v1` — minimal FASTQ preprocessing, reduced stage set.
- `fastq-to-fastq__adna__v1` — aDNA-oriented FASTQ defaults.

## fastq → bam
- `fastq-to-bam__default__v1` — FASTQ preprocess → align → BAM QC/damage (modern defaults).
- `fastq-to-bam__adna_shotgun__v1` — FASTQ preprocess → align → BAM QC/damage (aDNA shotgun).

## bam-only
- `bam-to-bam__default__v1` — standard BAM QC + damage assessments.
- `bam-to-bam__adna_shotgun__v1` — aDNA shotgun defaults.
- `bam-to-bam__adna_capture__v1` — aDNA capture defaults.

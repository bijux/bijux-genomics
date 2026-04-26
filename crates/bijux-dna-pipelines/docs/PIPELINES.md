# Pipelines

This is the authoritative human-readable inventory of pipeline profiles managed by `bijux-dna-pipelines`. The executable registry authority is `src/registry/catalog/pipeline_registry.rs`, with profile families assembled under `src/registry/families/`.

## Pipeline Model
A profile chooses a stable pipeline ID and declares stages, defaults, invariants, capabilities, and provenance. Defaults merge with explicit precedence:

```text
profile > pipeline > global
```

Profiles are deterministic contract data. They do not execute tools.

## FASTQ to FASTQ
- `fastq-to-fastq__adna__v1` — ancient-DNA FASTQ preprocessing with short-fragment handling.
- `fastq-to-fastq__default__v1` — standard modern FASTQ preprocessing.
- `fastq-to-fastq__minimal__v1` — reduced FASTQ preprocessing profile for minimal contracts.
- `fastq-to-fastq__reference_adna__v1` — reference-grade ancient-DNA FASTQ preprocessing.

## FASTQ to BAM
- `fastq-to-bam__adna_shotgun__v1` — FASTQ preprocessing plus BAM handoff for ancient-DNA shotgun workflows.
- `fastq-to-bam__default__v1` — FASTQ preprocessing plus BAM handoff for modern workflows.

## BAM to BAM
- `bam-to-bam__adna_capture__v1` — ancient-DNA capture BAM QC and damage profile.
- `bam-to-bam__adna_shotgun__v1` — ancient-DNA shotgun BAM QC and damage profile.
- `bam-to-bam__default__v1` — standard BAM QC and damage profile.
- `bam-to-bam__reference_adna__v1` — reference-grade ancient-DNA BAM profile.

## VCF to VCF
- `vcf-to-vcf__minimal__v1` — minimal VCF normalization and validation profile.
- `vcf-to-vcf__reference_basic__v1` — reference-oriented VCF normalization and annotation profile.

## Versioning
- Add a new pipeline ID when scientific intent changes.
- Add a new pipeline ID when default changes alter expected outputs or interpretation.
- Keep additive metadata changes on the existing ID only when downstream output semantics do not change.

## Add-Profile Checklist
- Add the profile builder in the owning domain or cross-domain module.
- Register the profile in `src/registry/families/`.
- Ensure lookup behavior is updated when the profile has direct lookup entrypoints.
- Update this inventory and `DEFAULTS_LEDGER.md` when defaults or provenance change.
- Add or update snapshots in `tests/snapshots/`.
- Run the contract and registry tests before committing.

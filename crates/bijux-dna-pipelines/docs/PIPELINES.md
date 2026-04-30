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
- `fastq-to-fastq__amplicon_standard__v1` — amplicon preprocessing with explicit primer, chimera, and ASV stages.
- `fastq-to-fastq__amplicon_umi__v1` — amplicon preprocessing with explicit UMI extraction before primer and chimera handling.
- `fastq-to-fastq__contaminant_depletion__v1` — shotgun preprocessing with governed reference-contaminant depletion.
- `fastq-to-fastq__default__v1` — standard modern FASTQ preprocessing.
- `fastq-to-fastq__edna_metabarcoding__v1` — eDNA/metabarcoding preprocessing with explicit screening and OTU-oriented branch semantics.
- `fastq-to-fastq__host_depletion__v1` — shotgun preprocessing with governed host-depletion.
- `fastq-to-fastq__minimal__v1` — reduced FASTQ preprocessing profile for minimal contracts.
- `fastq-to-fastq__qc_only__v1` — QC-only preprocessing that preserves raw reads while materializing governed reports.
- `fastq-to-fastq__reference_adna__v1` — reference-grade ancient-DNA FASTQ preprocessing.
- `fastq-to-fastq__rrna_depletion__v1` — shotgun preprocessing with governed rRNA-depletion.
- `fastq-to-fastq__trim_qc__v1` — production trim-and-QC preprocessing for modern FASTQ runs.
- `fastq-to-fastq__umi__v1` — shotgun preprocessing with explicit UMI extraction and downstream retention surfaces.

## FASTQ to BAM
- `fastq-to-bam__adna_shotgun__v1` — FASTQ preprocessing plus BAM handoff for ancient-DNA shotgun workflows.
- `fastq-to-bam__default__v1` — FASTQ preprocessing plus BAM handoff for modern workflows.

## FASTQ to VCF
- `fastq-to-vcf__minimal__v1` — tiny FASTQ preprocessing plus BAM alignment/genotyping handoff into VCF filtering and stats.

## BAM to BAM
- `bam-to-bam__adna_capture__v1` — ancient-DNA capture BAM QC and damage profile.
- `bam-to-bam__adna_shotgun__v1` — ancient-DNA shotgun BAM QC and damage profile.
- `bam-to-bam__default__v1` — standard BAM QC and damage profile.
- `bam-to-bam__reference_adna__v1` — reference-grade ancient-DNA BAM profile.

## BAM to VCF
- `bam-to-vcf__default__v1` — BAM QC/genotyping handoff into VCF filtering and stats with explicit BAM index and coverage prerequisites.

## VCF to VCF
- `vcf-to-vcf__minimal__v1` — minimal VCF normalization and validation profile.
- `vcf-to-vcf__reference_basic__v1` — reference-oriented VCF normalization and annotation profile.

## Workflow Templates
- `cross.fastq_to_bam_modern` — sample-sheet-aware FASTQ to BAM workflow template with governed layout, sample metadata, and reference admission checks.
- `cross.bam_to_vcf_default` — sample-sheet-aware BAM to VCF workflow template with explicit BAM index, coverage, and downstream failure policy.
- `cross.fastq_to_vcf_minimal` — tiny FASTQ to VCF workflow template with explicit validate/trim/align/call/stats ordering, fan-out/fan-in rules, and evidence-summary ordering.

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

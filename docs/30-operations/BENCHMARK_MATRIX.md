# Benchmark Matrix

Iteration-06 benchmark matrix workflow for FASTQ, BAM, VCF, and cross-domain bindings.

## Purpose

- Generate deterministic stage/tool rows from domain stage catalogs and registry bindings.
- Classify readiness from corpus, database, and image surface matching.
- Emit repetition policy per row to drive benchmark campaigns.

## Scope

- Applies to matrix generation for HPC campaign planning and dry-run readiness.
- Covers domain selectors, readiness classes, and optional strict refusal gating.

## Non-goals

- Does not submit jobs to Slurm.
- Does not mutate campaign configuration files.
- Does not replace scientific stage/tool contract validation.

## Contracts

- `cargo run -q -p bijux-dna -- config benchmark-matrix` must remain deterministic for identical inputs.
- Readiness classes must remain exactly `ready`, `degraded`, or `refuse`.
- `--fail-on-refuse` must exit non-zero when any row resolves to `refuse`.

## Command

```bash
cargo run -q -p bijux-dna -- config benchmark-matrix \
  --config benchmarks/configs/hpc/campaign/lunarc-small.toml \
  --domain all \
  --out artifacts/benchmark/matrix.json \
  --json
```

## Domain Selectors

- `--domain fastq`
- `--domain bam`
- `--domain vcf`
- `--domain cross`
- `--domain all`

## Readiness Classes

- `ready`: corpus/database/image matching succeeded.
- `degraded`: exactly one required surface is missing.
- `refuse`: two or more required surfaces are missing.

## Enforcement

Use `--fail-on-refuse` to fail command execution when any row is classified as `refuse`.

```bash
cargo run -q -p bijux-dna -- config benchmark-matrix \
  --config benchmarks/configs/hpc/campaign/lunarc-small.toml \
  --domain all \
  --fail-on-refuse
```

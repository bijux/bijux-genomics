# Benchmark Matrix

Iteration-06 benchmark matrix workflow for FASTQ, BAM, VCF, and cross-domain bindings.

## Purpose

- Generate deterministic stage/tool rows from domain stage catalogs and registry bindings.
- Classify readiness from corpus, database, and image surface matching.
- Emit repetition policy per row to drive benchmark campaigns.

## Command

```bash
cargo run -q -p bijux-dna -- config benchmark-matrix \
  --config configs/hpc/campaign/lunarc-small.toml \
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
  --config configs/hpc/campaign/lunarc-small.toml \
  --domain all \
  --fail-on-refuse
```

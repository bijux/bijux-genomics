# Bijux canonical FASTQ benchmark dataset (v1)

Status: **frozen**. Do not modify these files without a version bump.

This dataset is the single source of truth for:
- image QA
- benchmarks
- CI checks

## Files (gzipped FASTQ)

**Single-end (SE)**
- Path: `tests/data/fastq/canonical/BIJUX_SE_R1.fastq.gz`
- Reads: 2000

**Paired-end (PE)**
- Path: `tests/data/fastq/canonical/BIJUX_PE_R1.fastq.gz`
- Path: `tests/data/fastq/canonical/BIJUX_PE_R2.fastq.gz`
- Reads: 2000 per mate

## Required properties

The dataset intentionally includes:
- adapters (appended to a subset of reads)
- low-quality tails (paired with adapter tails)
- short reads (truncated subset)
- UMIs (embedded in read headers as `UMI:XXXX`)

These properties are deterministic and must remain present.

## SHA256 checksums

```
aa0d377ec155f3205f02fb4fa9cb9bc9f1216b15e1ae4e047679184ae1f53af2  tests/data/fastq/canonical/BIJUX_SE_R1.fastq.gz
ea09b95a1563c7cdf8b15d56318f2be224a9ec45697f1706291e442ee8293887  tests/data/fastq/canonical/BIJUX_PE_R1.fastq.gz
131c44a3052d518046d52f75bfa4745468cf77972bbfb04280c9c5b14149f540  tests/data/fastq/canonical/BIJUX_PE_R2.fastq.gz
```

## Contract

- Paths and filenames are stable.
- Hashes are immutable.
- Any change requires a new dataset version and documentation update.

<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::bam_observation::report -->
# bam.endogenous_content qualification benchmark

**Verdict: passed.** This campaign measures a reproducible primary mapped-read fraction for one BAM and one reference. It is an operational endogenous-content proxy, not biological truth about the library's endogenous DNA fraction.

## Scientific Scope

What fraction of primary reads in the governed ancient horse alignment map to the declared EquCab3 reference before duplicate removal?

Comparison scope: primary mapped-read fraction from one governed coordinate-sorted EquCab3 BAM.

Accuracy claim: `mapping_derived_fraction_not_endogenous_truth`.

## Reproducible Design

- Campaign: `run-2003`
- Repository commit: `27d88388e2fed29f96478c32ec5a5e8c73a7747d`
- Container catalog: `26eb025bbbdfe8a60e4f15f0832fb911605064ce1418d57c81d61a94bb66594b`
- Tool closure: `samtools`
- Repetitions: 1 warmup and 3 measured
- Corpus: `ancient-horse-alignment-qualification`
- Result contract: `endogenous-content.tsv`

## Comparison Closure

- Immutable inputs: `e2c4b15564206e9de17af5678d56a8ad298fdb70a56a1f29f20878cc846fbd80`
- Effective parameters: `0c47a97c87ae62e2b7f588bbdbc4caa8d03c15ecb92dadb3c5ca7c93b3290741`
- Repository source: `27d88388e2fed29f96478c32ec5a5e8c73a7747d`

## Performance

| Tool | Slurm job | Wall mean (range), s | CPU mean, s | Peak RSS mean, MiB | Input bytes | Output mean, bytes |
|---|---:|---:|---:|---:|---:|---:|
| `samtools` | `3473533` | 0.557 (0.520-0.620) | 0.540 | 45.6 | 26592222 | 731.0 |

## Scientific Observations

### samtools

| Metric | Measured repetitions |
|---|---|
| `mapped_fraction` | 0.996518910953, 0.996518910953, 0.996518910953 |
| `mapped_nonduplicate_primary_reads` | 235311.000000000000, 235311.000000000000, 235311.000000000000 |
| `mapped_primary_reads` | 235311.000000000000, 235311.000000000000, 235311.000000000000 |
| `total_primary_reads` | 236133.000000000000, 236133.000000000000, 236133.000000000000 |

The `endogenous-content.tsv` artifact was identical across measured repetitions by normalized endogenous-content TSV bytes: `217de08cebc5759bf8f216ba15686c4fb748e50adaefcb01edc65ad6fb625d06`.

Evidence: Slurm job `3473533`, node `cn135`, image `21252c2a23f5464381f5602a60eef97ee03394e4376b9c6ab5b643974a949673`, acceptance `68031e091ff1bda6c5dd47dcfc47de8b9627e610ffe8b4611e849072b154bba8`.

## Cross-tool Agreement

Agreement status: **not applicable**.

| Metric | Tool means | Observed range | Allowed range | Status |
|---|---|---:|---:|---|

## Limitations

- The result depends on the EquCab3 reference, upstream alignment parameters, and the BAM flag semantics declared by this campaign.
- Mapped-read fraction does not distinguish authentic ancient horse molecules from contamination that also maps to EquCab3.
- The corpus is one low-coverage ancient horse library and does not establish population-wide or protocol-wide behavior.

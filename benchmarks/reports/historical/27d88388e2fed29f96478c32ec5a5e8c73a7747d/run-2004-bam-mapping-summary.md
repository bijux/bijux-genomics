<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::bam_mapping_summary::report -->
# bam.mapping_summary qualification benchmark

**Verdict: passed.** This campaign compares Picard and Samtools primary-read accounting under one explicit aggregation contract. Exact agreement establishes interoperable BAM flag interpretation for this corpus; it does not establish that the upstream alignments are biologically correct.

## Scientific Scope

Do the retained BAM summary tools report the same primary-read and primary-mapped-read counts for a governed ancient horse alignment, and what resources does each implementation consume?

Comparison scope: primary mapping counts from one governed coordinate-sorted EquCab3 BAM.

Accuracy claim: `cross_tool_primary_mapping_summary_consistency_not_alignment_truth`.

## Reproducible Design

- Campaign: `run-2004`
- Repository commit: `27d88388e2fed29f96478c32ec5a5e8c73a7747d`
- Container catalog: `26eb025bbbdfe8a60e4f15f0832fb911605064ce1418d57c81d61a94bb66594b`
- Tool closure: `picard`, `samtools`
- Repetitions: 1 warmup and 3 measured
- Corpus: `ancient-horse-alignment-qualification`
- Result contract: `mapping-summary.tsv`

## Comparison Closure

- Immutable inputs: `e2c4b15564206e9de17af5678d56a8ad298fdb70a56a1f29f20878cc846fbd80`
- Effective parameters: `49d84b429b703d2d28158a4c17aabb5a002288e9f51eb7ecda5d54409476122a`
- Repository source: `27d88388e2fed29f96478c32ec5a5e8c73a7747d`

## Performance

| Tool | Slurm job | Wall mean (range), s | CPU mean, s | Peak RSS mean, MiB | Input bytes | Output mean, bytes |
|---|---:|---:|---:|---:|---:|---:|
| `picard` | `3473534` | 3.847 (3.760-4.020) | 3.800 | 667.3 | 26592222 | 5226.0 |
| `samtools` | `3473535` | 0.307 (0.300-0.310) | 0.290 | 45.9 | 26592222 | 1094.0 |

## Scientific Observations

### picard

| Metric | Measured repetitions |
|---|---|
| `mapped_primary_fraction` | 0.996518910953, 0.996518910953, 0.996518910953 |
| `mapped_primary_reads` | 235311.000000000000, 235311.000000000000, 235311.000000000000 |
| `paired_primary_reads` | 234450.000000000000, 234450.000000000000, 234450.000000000000 |
| `primary_reads` | 236133.000000000000, 236133.000000000000, 236133.000000000000 |

The `mapping-summary.tsv` artifact was identical across measured repetitions by normalized mapping-summary TSV bytes: `e9271883fe302da8df277f6cb22b65a239d9f9f01a0bb88f13dac7fc5f17e6e0`.

Evidence: Slurm job `3473534`, node `cn135`, image `5126331cdaf67eef217e5bee96292a33bf1f80b0922fa2a980e3c8e621471bc1`, acceptance `3aa30fdf101e8a89535d953b1f5d879c49abd696ab391543c64314fd0cac93c0`.

### samtools

| Metric | Measured repetitions |
|---|---|
| `mapped_primary_fraction` | 0.996518910953, 0.996518910953, 0.996518910953 |
| `mapped_primary_reads` | 235311.000000000000, 235311.000000000000, 235311.000000000000 |
| `paired_primary_reads` | 234450.000000000000, 234450.000000000000, 234450.000000000000 |
| `primary_reads` | 236133.000000000000, 236133.000000000000, 236133.000000000000 |

The `mapping-summary.tsv` artifact was identical across measured repetitions by normalized mapping-summary TSV bytes: `e9271883fe302da8df277f6cb22b65a239d9f9f01a0bb88f13dac7fc5f17e6e0`.

Evidence: Slurm job `3473535`, node `cn135`, image `21252c2a23f5464381f5602a60eef97ee03394e4376b9c6ab5b643974a949673`, acceptance `3a6e6a3253a72ff068040241d1a098ea9062c7245594bdbcdfe547999bf8b233`.

## Cross-tool Agreement

Agreement status: **within tolerance**.

| Metric | Tool means | Observed range | Allowed range | Status |
|---|---|---:|---:|---|
| `primary_reads` | `picard`=236133.000000000000, `samtools`=236133.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `mapped_primary_reads` | `picard`=235311.000000000000, `samtools`=235311.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `mapped_primary_fraction` | `picard`=0.996518910953, `samtools`=0.996518910953 | 0.000000000000 | 0.000000000998 | within |
| `paired_primary_reads` | `picard`=234450.000000000000, `samtools`=234450.000000000000 | 0.000000000000 | 0.000000000000 | within |

## Limitations

- The corpus is one low-coverage ancient horse library and cannot establish agreement across all aligners, sequencing technologies, or BAM flag combinations.
- Picard PAIR and UNPAIRED categories are deliberately aggregated to match Samtools primary-record semantics; FIRST_OF_PAIR and SECOND_OF_PAIR rows are excluded because including them would double-count paired records.
- Exact count agreement checks summary interoperability, not mapping accuracy against an independent alignment truth set.
- Both tools run with one allocated CPU because the measured summary operations do not expose equivalent useful parallelism.

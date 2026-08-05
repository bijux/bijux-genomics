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
- Repository commit: `48a75c5aaf233d5a39b43d987c019b3428a77949`
- Container catalog: `d6b352cec629b12396c93e67968a42b43f576550038cda25d16cc88db2639250`
- Tool closure: `picard`, `samtools`
- Repetitions: 1 warmup and 3 measured
- Corpus: `ancient-horse-alignment-qualification`
- Result contract: `mapping-summary.tsv`

## Comparison Closure

- Immutable inputs: `3dfe7e5aab7fddd3ac3b35abcb4748495b7c44de59bf32661f5916780d2fbd73`
- Effective parameters: `9b4c1bd2ebd4575e48de46aafa35d3045ba9ab122990eccda2761b71bdf3fa5f`
- Repository source: `48a75c5aaf233d5a39b43d987c019b3428a77949`

## Performance

| Tool | Slurm job | Wall mean (range), s | CPU mean, s | Peak RSS mean, MiB | Input bytes | Output mean, bytes |
|---|---:|---:|---:|---:|---:|---:|
| `picard` | `3461955` | 3.863 (3.700-3.960) | 3.817 | 667.8 | 26592222 | 5226.0 |
| `samtools` | `3461956` | 0.313 (0.310-0.320) | 0.297 | 42.5 | 26592222 | 1094.0 |

## Scientific Observations

### picard

| Metric | Measured repetitions |
|---|---|
| `mapped_primary_fraction` | 0.996518910953, 0.996518910953, 0.996518910953 |
| `mapped_primary_reads` | 235311.000000000000, 235311.000000000000, 235311.000000000000 |
| `paired_primary_reads` | 234450.000000000000, 234450.000000000000, 234450.000000000000 |
| `primary_reads` | 236133.000000000000, 236133.000000000000, 236133.000000000000 |

The `mapping-summary.tsv` artifact was identical across measured repetitions by normalized mapping-summary TSV bytes: `e9271883fe302da8df277f6cb22b65a239d9f9f01a0bb88f13dac7fc5f17e6e0`.

Evidence: Slurm job `3461955`, node `cn162`, image `649ebc98578bc1d3b4e5d233e5448da4a5908b1fcba44f1c432825cc0e576a49`, acceptance `f0a860be20fe9e2cb75db6b07bc8b5dff49dac29ae0fc7f938c8afebbbcee1a2`.

### samtools

| Metric | Measured repetitions |
|---|---|
| `mapped_primary_fraction` | 0.996518910953, 0.996518910953, 0.996518910953 |
| `mapped_primary_reads` | 235311.000000000000, 235311.000000000000, 235311.000000000000 |
| `paired_primary_reads` | 234450.000000000000, 234450.000000000000, 234450.000000000000 |
| `primary_reads` | 236133.000000000000, 236133.000000000000, 236133.000000000000 |

The `mapping-summary.tsv` artifact was identical across measured repetitions by normalized mapping-summary TSV bytes: `e9271883fe302da8df277f6cb22b65a239d9f9f01a0bb88f13dac7fc5f17e6e0`.

Evidence: Slurm job `3461956`, node `cn162`, image `a29122faccca9bcba0c51adc528400970502b32e0a57f54e5c15cadda879b85f`, acceptance `a01f0d3c31fb7dc30b3fe558425a5feea4248febac9f57d0a4a2dc4ed76e17b7`.

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

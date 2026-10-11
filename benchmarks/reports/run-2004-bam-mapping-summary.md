<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: benchmarks/benchmark_evidence.py -->
# bam.mapping_summary qualification

**Verdict: passed.** This campaign compares Picard and Samtools primary-read accounting under one explicit aggregation contract. Exact agreement establishes interoperable BAM flag interpretation for this corpus; it does not establish that the upstream alignments are biologically correct.

This is qualification evidence. It proves the declared execution and metric contract on the named data, but it is not a comparative scientific benchmark and must not be used to rank tools.

## Scientific scope

Do the retained BAM summary tools report the same primary-read and primary-mapped-read counts for a governed ancient horse alignment, and what resources does each implementation consume?

Comparison scope: primary mapping counts from one governed coordinate-sorted EquCab3 BAM.

Accuracy claim: `cross_tool_primary_mapping_summary_consistency_not_alignment_truth`.

## Design

- Campaign: `run-2004`
- Evidence tier: `qualification`
- Scientific profile: `bam-observational-qc`
- Tool closure: `picard`, `samtools`
- Execution: apptainer under slurm, container-only
- Threads by tool: `picard`=1, `samtools`=1
- Cache policy: warm
- Repetitions: 1 warmup and 3 measured
- Corpus: `ena-prjeb22390-err2112767-bam`
- Corpus portfolio: `ancient-horse-alignment-qualification`
- Reference: none
- Databases: none
- Repeated result artifact: `mapping-summary.tsv`

## Performance

| Tool | Wall mean (range), s | CPU mean, s | Peak RSS mean (range), MiB | Output mean, bytes |
|---|---:|---:|---:|---:|
| `picard` | 4.777 (4.670–4.840) | 4.730 | 652.2 (652.0–652.3) | 4769.0 |
| `samtools` | 0.120 (0.120–0.120) | 0.117 | 9.2 (9.0–9.4) | 635.0 |

## Scientific observations

### picard

| Metric | Measured repetitions |
|---|---|
| `primary_reads` | 236133, 236133, 236133 |
| `mapped_primary_reads` | 235311, 235311, 235311 |
| `mapped_primary_fraction` | 0.996518910953, 0.996518910953, 0.996518910953 |
| `paired_primary_reads` | 234450, 234450, 234450 |

The `mapping-summary.tsv` result was byte-identical across all repetitions: `e9271883fe302da8df277f6cb22b65a239d9f9f01a0bb88f13dac7fc5f17e6e0`.

Evidence: Slurm job `3459931`, repository `e1c74a4d910ae0fcf0855a0b776830188ea2eed4`, image `649ebc98578bc1d3b4e5d233e5448da4a5908b1fcba44f1c432825cc0e576a49`.

### samtools

| Metric | Measured repetitions |
|---|---|
| `primary_reads` | 236133, 236133, 236133 |
| `mapped_primary_reads` | 235311, 235311, 235311 |
| `mapped_primary_fraction` | 0.996518910953, 0.996518910953, 0.996518910953 |
| `paired_primary_reads` | 234450, 234450, 234450 |

The `mapping-summary.tsv` result was byte-identical across all repetitions: `e9271883fe302da8df277f6cb22b65a239d9f9f01a0bb88f13dac7fc5f17e6e0`.

Evidence: Slurm job `3459932`, repository `e1c74a4d910ae0fcf0855a0b776830188ea2eed4`, image `a29122faccca9bcba0c51adc528400970502b32e0a57f54e5c15cadda879b85f`.

## Cross-tool agreement

Agreement status: **within tolerance**.

| Metric | Tool means | Observed range | Allowed range | Status |
|---|---|---:|---:|---|
| `primary_reads` | `picard`=236133, `samtools`=236133 | 0 | 0 | within |
| `mapped_primary_reads` | `picard`=235311, `samtools`=235311 | 0 | 0 | within |
| `mapped_primary_fraction` | `picard`=0.996518910953, `samtools`=0.996518910953 | 0 | 9.97518910953e-10 | within |
| `paired_primary_reads` | `picard`=234450, `samtools`=234450 | 0 | 0 | within |

## Limitations

- The corpus is one low-coverage ancient horse library and cannot establish agreement across all aligners, sequencing technologies, or BAM flag combinations.
- Picard PAIR and UNPAIRED categories are deliberately aggregated to match Samtools primary-record semantics; FIRST_OF_PAIR and SECOND_OF_PAIR rows are excluded because including them would double-count paired records.
- Exact count agreement checks summary interoperability, not mapping accuracy against an independent alignment truth set.
- Both tools run with one allocated CPU because the measured summary operations do not expose equivalent useful parallelism.

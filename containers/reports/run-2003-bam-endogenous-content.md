<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: containers/benchmarks/benchmark_evidence.py -->
# bam.endogenous_content benchmark

**Verdict: passed.** This campaign measures a reproducible primary mapped-read fraction for one BAM and one reference. It is an operational endogenous-content proxy, not biological truth about the library's endogenous DNA fraction.

## Scientific scope

What fraction of primary reads in the governed ancient horse alignment map to the declared EquCab3 reference before duplicate removal?

Comparison scope: primary mapped-read fraction from one governed coordinate-sorted EquCab3 BAM.

Accuracy claim: `mapping_derived_fraction_not_endogenous_truth`.

## Design

- Campaign: `run-2003`
- Tool closure: `samtools`
- Execution: apptainer under slurm, container-only
- Threads by tool: `samtools`=4
- Cache policy: warm
- Repetitions: 1 warmup and 3 measured
- Corpus: `ena-prjeb22390-err2112767-bam`
- Reference: `equcab3-gcf-002863925.1`
- Databases: none
- Repeated result artifact: `endogenous-content.tsv`

## Performance

| Tool | Wall mean (range), s | CPU mean, s | Peak RSS mean (range), MiB | Output mean, bytes |
|---|---:|---:|---:|---:|
| `samtools` | 0.203 (0.200–0.210) | 0.390 | 10.2 (10.1–10.5) | 140.0 |

## Scientific observations

### samtools

| Metric | Measured repetitions |
|---|---|
| `total_primary_reads` | 236133, 236133, 236133 |
| `mapped_primary_reads` | 235311, 235311, 235311 |
| `mapped_fraction` | 0.996518910953, 0.996518910953, 0.996518910953 |
| `mapped_nonduplicate_primary_reads` | 235311, 235311, 235311 |

The `endogenous-content.tsv` result was byte-identical across all repetitions: `217de08cebc5759bf8f216ba15686c4fb748e50adaefcb01edc65ad6fb625d06`.

Evidence: Slurm job `3459847`, repository `4e24ded282af44fb985dd869ba944684538814aa`, image `a29122faccca9bcba0c51adc528400970502b32e0a57f54e5c15cadda879b85f`.

## Cross-tool agreement

This campaign has one tool, so cross-tool agreement is not applicable.

## Limitations

- The result depends on the EquCab3 reference, upstream alignment parameters, and the BAM flag semantics declared by this campaign.
- Mapped-read fraction does not distinguish authentic ancient horse molecules from contamination that also maps to EquCab3.
- The corpus is one low-coverage ancient horse library and does not establish population-wide or protocol-wide behavior.

<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: benchmarks/benchmark_evidence.py -->
# bam.coverage qualification

**Verdict: passed.** This campaign compares resource use, repeatability, and agreement of coverage summaries over one real ancient horse BAM. Agreement is a consistency check, not an accuracy claim against an independent coverage truth set.

This is qualification evidence. It proves the declared execution and metric contract on the named data, but it is not a comparative scientific benchmark and must not be used to rank tools.

## Scientific scope

How efficiently and consistently do the retained BAM coverage tools summarize reference-wide breadth and depth for the same real low-coverage ancient horse alignment?

Comparison scope: whole-reference coverage summaries from one governed coordinate-sorted EquCab3 BAM.

Accuracy claim: `cross_tool_coverage_summary_consistency_not_truth_accuracy`.

## Design

- Campaign: `run-2002`
- Evidence tier: `qualification`
- Scientific profile: `bam-observational-qc`
- Tool closure: `bedtools`, `mosdepth`, `samtools`
- Execution: apptainer under slurm, container-only
- Threads by tool: `bedtools`=1, `mosdepth`=4, `samtools`=1
- Cache policy: warm
- Repetitions: 1 warmup and 3 measured
- Corpus: `ena-prjeb22390-err2112767-bam`
- Corpus portfolio: `ancient-horse-alignment-qualification`
- Reference: `equcab3-gcf-002863925.1`
- Databases: none
- Repeated result artifact: `coverage-summary.tsv`

## Performance

| Tool | Wall mean (range), s | CPU mean, s | Peak RSS mean (range), MiB | Output mean, bytes |
|---|---:|---:|---:|---:|
| `bedtools` | 70.157 (69.880–70.340) | 69.810 | 1442.1 (1442.1–1442.2) | 234842.0 |
| `mosdepth` | 13.800 (13.760–13.830) | 14.887 | 1851.6 (1724.9–2104.8) | 425471.0 |
| `samtools` | 0.760 (0.760–0.760) | 0.750 | 9.5 (9.4–9.8) | 195906.0 |

## Scientific observations

### bedtools

| Metric | Measured repetitions |
|---|---|
| `reference_bases` | 2506966135, 2506966135, 2506966135 |
| `covered_bases` | 23099381, 23099381, 23099381 |
| `coverage_fraction` | 0.009214077796, 0.009214077796, 0.009214077796 |
| `mean_depth` | 0.009331205026, 0.009331205026, 0.009331205026 |

The `coverage-summary.tsv` result was byte-identical across all repetitions: `68ad5329418c08716d19b4de93f0c932f78b575392f23e526f55e7aa0466bf7f`.

Evidence: Slurm job `3459834`, repository `b087cd5ca6412b19a51cb43e7bf4eb098235b7db`, image `fe469da8f256a3a5730207531c03886b72675fecadbed2b3979c29e34634597e`.

### mosdepth

| Metric | Measured repetitions |
|---|---|
| `reference_bases` | 2506966135, 2506966135, 2506966135 |
| `covered_bases` | 23099381, 23099381, 23099381 |
| `coverage_fraction` | 0.009214077796, 0.009214077796, 0.009214077796 |
| `mean_depth` | 0.009331205026, 0.009331205026, 0.009331205026 |

The `coverage-summary.tsv` result was byte-identical across all repetitions: `68ad5329418c08716d19b4de93f0c932f78b575392f23e526f55e7aa0466bf7f`.

Evidence: Slurm job `3459839`, repository `b087cd5ca6412b19a51cb43e7bf4eb098235b7db`, image `8bd0a039c887f56a6fa9d68e0ac94edfc1177843bc1321eb5e941a1f516dde04`.

### samtools

| Metric | Measured repetitions |
|---|---|
| `reference_bases` | 2506966135, 2506966135, 2506966135 |
| `covered_bases` | 23099381, 23099381, 23099381 |
| `coverage_fraction` | 0.009214077796, 0.009214077796, 0.009214077796 |
| `mean_depth` | 0.009331205928, 0.009331205928, 0.009331205928 |

The `coverage-summary.tsv` result was byte-identical across all repetitions: `06290c6c481cecdf70926fd7d2597490bbfdb4e3789b7e8af6838445e8c42783`.

Evidence: Slurm job `3459793`, repository `b087cd5ca6412b19a51cb43e7bf4eb098235b7db`, image `a29122faccca9bcba0c51adc528400970502b32e0a57f54e5c15cadda879b85f`.

## Cross-tool agreement

Agreement status: **within tolerance**.

| Metric | Tool means | Observed range | Allowed range | Status |
|---|---|---:|---:|---|
| `reference_bases` | `bedtools`=2506966135, `mosdepth`=2506966135, `samtools`=2506966135 | 0 | 0 | within |
| `covered_bases` | `bedtools`=23099381, `mosdepth`=23099381, `samtools`=23099381 | 0 | 1155969.05 | within |
| `coverage_fraction` | `bedtools`=0.009214077796, `mosdepth`=0.009214077796, `samtools`=0.009214077796 | 0 | 0.0004617038898 | within |
| `mean_depth` | `bedtools`=0.009331205026, `mosdepth`=0.009331205026, `samtools`=0.009331205928 | 9.02000000733e-10 | 0.0005665602964 | within |

## Limitations

- The corpus is one low-coverage ancient horse library and does not represent high-depth, long-read, or cohort workloads.
- Cross-tool agreement can reveal integration defects but does not establish accuracy without an independent truth coverage surface.
- Warm-cache repetitions measure stable execution after one warmup and exclude cold shared-filesystem startup.

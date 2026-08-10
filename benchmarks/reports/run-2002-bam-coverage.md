<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::bam_observation::report -->
# bam.coverage qualification benchmark

**Verdict: passed.** This campaign compares resource use, repeatability, and agreement of coverage summaries over one real ancient horse BAM. Agreement is a consistency check, not an accuracy claim against an independent coverage truth set.

## Scientific Scope

How efficiently and consistently do the retained BAM coverage tools summarize reference-wide breadth and depth for the same real low-coverage ancient horse alignment?

Comparison scope: whole-reference coverage summaries from one governed coordinate-sorted EquCab3 BAM.

Accuracy claim: `cross_tool_coverage_summary_consistency_not_truth_accuracy`.

## Reproducible Design

- Campaign: `run-2002`
- Repository commit: `8282c2232d021ce60f07edb1ddc65d85598836ed`
- Container catalog: `7dc269495d5dcb614763e84b13751b1f7f5f7ff8f2c7d66c7a3485b63c8ac904`
- Tool closure: `bedtools`, `mosdepth`, `samtools`
- Repetitions: 1 warmup and 3 measured
- Corpus: `ancient-horse-alignment-qualification`
- Result contract: `coverage-summary.tsv`

## Comparison Closure

- Immutable inputs: `ad74091a4e1236479827d192c9f681d6883bdc4481d3ff8b7f8dc7ed9c584645`
- Effective parameters: `e0e9eaf5b0f94543f61d21939a0071913b2aef48b62f0da6afe43907f88d2229`
- Repository source: `8282c2232d021ce60f07edb1ddc65d85598836ed`

## Performance

| Tool | Slurm job | Wall mean (range), s | CPU mean, s | Peak RSS mean, MiB | Input bytes | Output mean, bytes |
|---|---:|---:|---:|---:|---:|---:|
| `bedtools` | `3473795` | 7.577 (7.540-7.630) | 7.480 | 1442.0 | 26592222 | 15727761.0 |
| `mosdepth` | `3473796` | 10.750 (10.670-10.860) | 12.063 | 1470.0 | 26592222 | 18339534.0 |
| `samtools` | `3473797` | 8.900 (8.750-9.180) | 8.807 | 43.1 | 26592222 | 529493018.0 |

## Scientific Observations

### bedtools

| Metric | Measured repetitions |
|---|---|
| `coverage_fraction` | 0.009214077796, 0.009214077796, 0.009214077796 |
| `covered_bases` | 23099381.000000000000, 23099381.000000000000, 23099381.000000000000 |
| `mean_depth` | 0.009331205026, 0.009331205026, 0.009331205026 |
| `reference_bases` | 2506966135.000000000000, 2506966135.000000000000, 2506966135.000000000000 |

The `coverage-summary.tsv` artifact was identical across measured repetitions by normalized coverage-summary TSV bytes: `68ad5329418c08716d19b4de93f0c932f78b575392f23e526f55e7aa0466bf7f`.

Evidence: Slurm job `3473795`, node `cn160`, image `b906fb291cd8089dda88aea35414aab274b447c382cda5bffaaa2e03c84805d0`, acceptance `34dbf71b48f3d81d0a3d245fed9de8679e7d3ed02b4785d299cacccf24c2df30`.

### mosdepth

| Metric | Measured repetitions |
|---|---|
| `coverage_fraction` | 0.009214077796, 0.009214077796, 0.009214077796 |
| `covered_bases` | 23099381.000000000000, 23099381.000000000000, 23099381.000000000000 |
| `mean_depth` | 0.009331205026, 0.009331205026, 0.009331205026 |
| `reference_bases` | 2506966135.000000000000, 2506966135.000000000000, 2506966135.000000000000 |

The `coverage-summary.tsv` artifact was identical across measured repetitions by normalized coverage-summary TSV bytes: `68ad5329418c08716d19b4de93f0c932f78b575392f23e526f55e7aa0466bf7f`.

Evidence: Slurm job `3473796`, node `cn160`, image `255f80faa0209197b9d5166ad25a97b42aa49e5144302c77bdd32b64fdea1397`, acceptance `555a3561ff146913aeb21613214a0c5596df3e6cc957abbd2bd414432b408b52`.

### samtools

| Metric | Measured repetitions |
|---|---|
| `coverage_fraction` | 0.009216767900, 0.009216767900, 0.009216767900 |
| `covered_bases` | 23106125.000000000000, 23106125.000000000000, 23106125.000000000000 |
| `mean_depth` | 0.009331205026, 0.009331205026, 0.009331205026 |
| `reference_bases` | 2506966135.000000000000, 2506966135.000000000000, 2506966135.000000000000 |

The `coverage-summary.tsv` artifact was identical across measured repetitions by normalized coverage-summary TSV bytes: `c82718e63db053e2ffec82829417fd7d0843b615b226a2d1817a82a2f24fb6f9`.

Evidence: Slurm job `3473797`, node `cn160`, image `21252c2a23f5464381f5602a60eef97ee03394e4376b9c6ab5b643974a949673`, acceptance `5fb4c5d5dbd0023260cf4096af609fc1cb238dd95c641bfb4263f7cf77cb64f1`.

## Cross-tool Agreement

Agreement status: **within tolerance**.

| Metric | Tool means | Observed range | Allowed range | Status |
|---|---|---:|---:|---|
| `reference_bases` | `bedtools`=2506966135.000000000000, `mosdepth`=2506966135.000000000000, `samtools`=2506966135.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `covered_bases` | `bedtools`=23099381.000000000000, `mosdepth`=23099381.000000000000, `samtools`=23106125.000000000000 | 6744.000000000000 | 1156306.250000000000 | within |
| `coverage_fraction` | `bedtools`=0.009214077796, `mosdepth`=0.009214077796, `samtools`=0.009216767900 | 0.000002690104 | 0.000461838395 | within |
| `mean_depth` | `bedtools`=0.009331205026, `mosdepth`=0.009331205026, `samtools`=0.009331205026 | 0.000000000000 | 0.000566560251 | within |

## Limitations

- The corpus is one low-coverage ancient horse library and does not represent high-depth, long-read, or cohort workloads.
- Cross-tool agreement can reveal integration defects but does not establish accuracy without an independent truth coverage surface.
- Warm-cache repetitions measure stable execution after one warmup and exclude cold shared-filesystem startup.

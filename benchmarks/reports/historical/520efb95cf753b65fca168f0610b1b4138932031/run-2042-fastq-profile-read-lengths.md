<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::fastq_profile_read_lengths::report -->
# fastq.profile_read_lengths comparative benchmark

**Verdict: passed.** Exact row agreement with an independent FASTQ decoder establishes record coverage and read-length correctness for this cohort. It does not assess read transformation, biological representativeness, or every valid FASTQ encoding.

## Scientific Scope

Do both exact read-length backends recover every independently decoded per-record length across the complete real human short-read diversity cohort, and what resources does each implementation consume?

Comparison scope: thirty FASTQ files from twenty ancient and modern human runs spanning single- and paired-end libraries.

Accuracy claim: `exact_per_record_length_concordance_with_independent_fastq_decode`.

## Reproducible Design

- Campaign: `run-2042`
- Repository commit: `520efb95cf753b65fca168f0610b1b4138932031`
- Container catalog: `d6b352cec629b12396c93e67968a42b43f576550038cda25d16cc88db2639250`
- Tool closure: `seqfu`, `seqkit`
- Repetitions: 1 warmup and 3 measured
- Corpus: `human-short-read-diversity`
- Result contract: `read-lengths.json`

## Comparison Closure

- Immutable inputs: `9da50b9e49370be6d7ffe55160dfe964bf53dd31bcc1cc22d1253ed17fd6c0c9`
- Effective parameters: `b04b7fdaa89b9657fb007d10a26101351bd33d1601e6915295a13d806183d6ad`
- Repository source: `520efb95cf753b65fca168f0610b1b4138932031`

## Performance

| Tool | Slurm job | Wall mean (range), s | CPU mean, s | Peak RSS mean, MiB | Throughput, MiB/s | Reads/s | Output mean, bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| `seqfu` | `3461973` | 582.330 (577.040-587.270) | 576.540 | 45.6 | 5.96 | 129101.9 | 1592964957.0 |
| `seqkit` | `3461974` | 119.797 (119.320-120.090) | 152.087 | 56.4 | 28.96 | 627535.0 | 1592964958.0 |

## Scientific Observations

### seqfu

| Metric | Measured repetitions |
|---|---|
| `distinct_read_lengths` | 287.000000000000, 287.000000000000, 287.000000000000 |
| `file_count` | 30.000000000000, 30.000000000000, 30.000000000000 |
| `histogram_sha256` | `054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5`, `054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5`, `054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5` |
| `maximum_read_length` | 301.000000000000, 301.000000000000, 301.000000000000 |
| `mean_read_length` | 60.979869758162, 60.979869758162, 60.979869758162 |
| `median_read_length` | 36.000000000000, 36.000000000000, 36.000000000000 |
| `minimum_read_length` | 15.000000000000, 15.000000000000, 15.000000000000 |
| `read_count` | 75175997.000000000000, 75175997.000000000000, 75175997.000000000000 |

The `read-lengths.json` artifact was identical across measured repetitions by canonical bundle content with repetition-local histogram paths normalized: `c6e4771f8239b8f77a46fd137a34ccf03646ce4b99ce5722a5f82d2ea5e9bc08`.

Evidence: Slurm job `3461973`, node `cn162`, image `ba6d5e5acd51137d94fee735ae5a5ffcde3f1841d675ef5bc9c131bb673b47c4`, acceptance `e4bcb2f7b89b008f0b5ca9de2bcbeb35d4357bf6b40b3ca7f59564e334f7bee3`.

### seqkit

| Metric | Measured repetitions |
|---|---|
| `distinct_read_lengths` | 287.000000000000, 287.000000000000, 287.000000000000 |
| `file_count` | 30.000000000000, 30.000000000000, 30.000000000000 |
| `histogram_sha256` | `054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5`, `054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5`, `054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5` |
| `maximum_read_length` | 301.000000000000, 301.000000000000, 301.000000000000 |
| `mean_read_length` | 60.979869758162, 60.979869758162, 60.979869758162 |
| `median_read_length` | 36.000000000000, 36.000000000000, 36.000000000000 |
| `minimum_read_length` | 15.000000000000, 15.000000000000, 15.000000000000 |
| `read_count` | 75175997.000000000000, 75175997.000000000000, 75175997.000000000000 |

The `read-lengths.json` artifact was identical across measured repetitions by canonical bundle content with repetition-local histogram paths normalized: `312d1fc4fe44184427fa83057dd7dc362db6db822e618446f93eace3f284f9b7`.

Evidence: Slurm job `3461974`, node `cn162`, image `4646ad26f8152674cda466d229ba0a5a495b9176ee4f3fc77f4d50cdf7e9d3d5`, acceptance `be1b7289a1d478c04f59da755dc56f83b294e8d947dc9adf13d7f241d6649f32`.

## Cross-tool Agreement

Agreement status: **within tolerance**.

| Metric | Tool means | Observed range | Allowed range | Status |
|---|---|---:|---:|---|
| `file_count` | `seqfu`=30.000000000000, `seqkit`=30.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `read_count` | `seqfu`=75175997.000000000000, `seqkit`=75175997.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `minimum_read_length` | `seqfu`=15.000000000000, `seqkit`=15.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `mean_read_length` | `seqfu`=60.979869758162, `seqkit`=60.979869758162 | 0.000000000000 | 0.000000000000 | within |
| `median_read_length` | `seqfu`=36.000000000000, `seqkit`=36.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `maximum_read_length` | `seqfu`=301.000000000000, `seqkit`=301.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `distinct_read_lengths` | `seqfu`=287.000000000000, `seqkit`=287.000000000000 | 0.000000000000 | 0.000000000000 | within |

## Limitations

- The cohort is curated for workload diversity and is not population-representative.
- The independent truth surface validates lengths and record closure but not sequence identity beyond the preserved record identifiers.
- Warm-cache repetitions characterize stable parsing after one warmup and do not measure cold shared-filesystem startup.
- SeqFu tabulate is single-stream; its one-CPU allocation is intentional and recorded rather than falsely projecting the shared thread override into unsupported backend flags.

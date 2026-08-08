<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::fastq_profile_reads::report -->
# fastq.profile_reads comparative benchmark

**Verdict: passed.** Exact agreement with the read and base counts produced during independent byte-level FASTQ admission establishes parsing correctness for this cohort. It does not validate transformations, biological representativeness, or unobserved FASTQ encodings.

## Scientific Scope

Do all retained FASTQ profiling tools recover the independently admitted read and base counts across the complete real human short-read diversity cohort, and what resources does each implementation consume?

Comparison scope: thirty FASTQ files from twenty ancient and modern human runs spanning single- and paired-end libraries.

Accuracy claim: `read_and_base_count_concordance_with_independent_admission_counts`.

## Reproducible Design

- Campaign: `run-2043`
- Repository commit: `520efb95cf753b65fca168f0610b1b4138932031`
- Container catalog: `d6b352cec629b12396c93e67968a42b43f576550038cda25d16cc88db2639250`
- Tool closure: `seqfu`, `seqkit`, `seqkit_stats`
- Repetitions: 1 warmup and 3 measured
- Corpus: `human-short-read-diversity`
- Result contract: `read-profile.tsv`

## Comparison Closure

- Immutable inputs: `687837050ebb8e197ae869a65718f9810a5e3d85bcc6d8a7ce12ba7e1d0da7ca`
- Effective parameters: `9e3ced20b09e554d7e34710ba96da65b7666aacc3a2f7a4725fe2aff72603d92`
- Repository source: `520efb95cf753b65fca168f0610b1b4138932031`

## Performance

| Tool | Slurm job | Wall mean (range), s | CPU mean, s | CPU use, % | Peak RSS mean, MiB | Throughput, MiB/s | Output mean, bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| `seqfu` | `3461976` | 38.700 (38.370-38.920) | 75.323 | 194.6 | 44.3 | 89.66 | 7141.0 |
| `seqkit` | `3461977` | 22.130 (21.980-22.230) | 74.767 | 337.9 | 71.0 | 156.78 | 8263.0 |
| `seqkit_stats` | `3461978` | 22.303 (22.210-22.410) | 75.017 | 336.3 | 70.3 | 155.56 | 8269.0 |

## Scientific Observations

### seqfu

| Metric | Measured repetitions |
|---|---|
| `base_count` | 4584222506.000000000000, 4584222506.000000000000, 4584222506.000000000000 |
| `file_count` | 30.000000000000, 30.000000000000, 30.000000000000 |
| `maximum_read_length` | 301.000000000000, 301.000000000000, 301.000000000000 |
| `minimum_read_length` | 15.000000000000, 15.000000000000, 15.000000000000 |
| `read_count` | 75175997.000000000000, 75175997.000000000000, 75175997.000000000000 |

The `read-profile.tsv` artifact was identical across measured repetitions by normalized read-profile TSV bytes: `66f32cc8526486ab5096ac77f6699316db231233220705a397ab14638fd07a89`.

Evidence: Slurm job `3461976`, node `cn162`, image `ba6d5e5acd51137d94fee735ae5a5ffcde3f1841d675ef5bc9c131bb673b47c4`, acceptance `6da6ec7dc088fb56b9242a2ba7b528a4bd5f9aa71bd5330b24e797dc33170084`.

### seqkit

| Metric | Measured repetitions |
|---|---|
| `base_count` | 4584222506.000000000000, 4584222506.000000000000, 4584222506.000000000000 |
| `file_count` | 30.000000000000, 30.000000000000, 30.000000000000 |
| `maximum_read_length` | 301.000000000000, 301.000000000000, 301.000000000000 |
| `minimum_read_length` | 15.000000000000, 15.000000000000, 15.000000000000 |
| `read_count` | 75175997.000000000000, 75175997.000000000000, 75175997.000000000000 |

The `read-profile.tsv` artifact was identical across measured repetitions by normalized read-profile TSV bytes: `66f32cc8526486ab5096ac77f6699316db231233220705a397ab14638fd07a89`.

Evidence: Slurm job `3461977`, node `cn162`, image `4646ad26f8152674cda466d229ba0a5a495b9176ee4f3fc77f4d50cdf7e9d3d5`, acceptance `e694f09c685a97b2a7e80cb6bb3b4cde419804cb8c766dece93afc5bf3c25866`.

### seqkit_stats

| Metric | Measured repetitions |
|---|---|
| `base_count` | 4584222506.000000000000, 4584222506.000000000000, 4584222506.000000000000 |
| `file_count` | 30.000000000000, 30.000000000000, 30.000000000000 |
| `maximum_read_length` | 301.000000000000, 301.000000000000, 301.000000000000 |
| `minimum_read_length` | 15.000000000000, 15.000000000000, 15.000000000000 |
| `read_count` | 75175997.000000000000, 75175997.000000000000, 75175997.000000000000 |

The `read-profile.tsv` artifact was identical across measured repetitions by normalized read-profile TSV bytes: `66f32cc8526486ab5096ac77f6699316db231233220705a397ab14638fd07a89`.

Evidence: Slurm job `3461978`, node `cn162`, image `315f69cd42259f5ef810ff0e9ea5ea6faa28ba8e2c3b6cc65b338d5ac2f4721c`, acceptance `b0338459136fd0e85ca14185e408fb12862913c11f613a45e330ae0182842236`.

## Cross-tool Agreement

Agreement status: **within tolerance**.

| Metric | Tool means | Observed range | Allowed range | Status |
|---|---|---:|---:|---|
| `file_count` | `seqfu`=30.000000000000, `seqkit`=30.000000000000, `seqkit_stats`=30.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `read_count` | `seqfu`=75175997.000000000000, `seqkit`=75175997.000000000000, `seqkit_stats`=75175997.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `base_count` | `seqfu`=4584222506.000000000000, `seqkit`=4584222506.000000000000, `seqkit_stats`=4584222506.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `minimum_read_length` | `seqfu`=15.000000000000, `seqkit`=15.000000000000, `seqkit_stats`=15.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `maximum_read_length` | `seqfu`=301.000000000000, `seqkit`=301.000000000000, `seqkit_stats`=301.000000000000 | 0.000000000000 | 0.000000000000 | within |

## Limitations

- The cohort is curated for workload diversity and is not population-representative.
- Minimum and maximum read lengths are cross-tool consistency observations because the admission catalog records reads and bases but not length extrema.
- Warm-cache repetitions characterize stable parsing after one warmup and do not measure cold shared-filesystem startup.
- The campaign covers the admitted gzip-compressed Illumina FASTQ files and does not generalize to every valid FASTQ encoding or compression format.

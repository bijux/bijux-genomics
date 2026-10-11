<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: benchmarks/benchmark_evidence.py -->
# fastq.profile_reads comparative

**Verdict: passed.** Exact agreement with the read and base counts produced during independent byte-level FASTQ admission establishes parsing correctness for this cohort. It does not validate transformations, biological representativeness, or unobserved FASTQ encodings.

This is comparative evidence governed by the declared scientific profile.

## Scientific scope

Do all retained FASTQ profiling tools recover the independently admitted read and base counts across the complete real human short-read diversity cohort, and what resources does each implementation consume?

Comparison scope: thirty FASTQ files from twenty ancient and modern human runs spanning single- and paired-end libraries.

Accuracy claim: `read_and_base_count_concordance_with_independent_admission_counts`.

## Design

- Campaign: `run-2043`
- Evidence tier: `comparative`
- Scientific profile: `fastq-observational-qc`
- Tool closure: `seqfu`, `seqkit`, `seqkit_stats`
- Execution: apptainer under slurm, container-only
- Threads by tool: `seqfu`=4, `seqkit`=4, `seqkit_stats`=4
- Cache policy: warm
- Repetitions: 1 warmup and 3 measured
- Corpus: `human-short-read-diversity`
- Corpus portfolio: `human-short-read-diversity`
- Reference: none
- Databases: none
- Repeated result artifact: `read-profile.tsv`

## Replay and comparison interface

- Repository commit: `0250599effd8ddcd48870d3ae717589ee71756f3`
- Immutable input closure: `6d1028d3a54e91f569129dfa735c0c94960dbcd19999279979643cb4ec9f44a7`
- Effective parameter closure: `bf06cbbab584742bc846c1474503d6760dbe690f80e4d6960199e052fea24534`
- Accepted inputs: `bijux.fastq.artifact.single_end.v1`, `bijux.fastq.artifact.paired_end.v1`, `bijux.fastq.artifact.merged.v1`
- Normalized output: `bijux.benchmark.fastq.profile_reads.bundle.v1`
- Output schema: `benchmarks/contracts/schemas/fastq-profile-reads-result.schema.json`
- Tool-independent consumer: `benchmark.fastq.profile_reads.compare`
- Substitution scope: normalized benchmark comparison only; this run does not claim operational downstream pipeline substitution

## Performance

| Tool | Wall mean (range), s | CPU mean, s | CPU use, % | Peak RSS mean (range), MiB | Throughput, MiB/s | Filesystem input/output operations | Output mean, bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| `seqfu` | 40.017 (38.929–42.052) | 76.892 | 192.3 | 45.1 (44.8–45.5) | 86.81 | 144.0/0.0 | 7141.0 |
| `seqkit` | 22.437 (22.380–22.494) | 75.540 | 336.7 | 65.1 (64.9–65.1) | 154.64 | 0.0/0.0 | 8263.0 |
| `seqkit_stats` | 22.172 (22.108–22.214) | 75.065 | 338.6 | 64.6 (64.4–64.8) | 156.48 | 0.0/0.0 | 8269.0 |

## Scientific observations

### seqfu

| Metric | Measured repetitions |
|---|---|
| `file_count` | 30, 30, 30 |
| `read_count` | 75175997, 75175997, 75175997 |
| `base_count` | 4584222506, 4584222506, 4584222506 |
| `minimum_read_length` | 15, 15, 15 |
| `maximum_read_length` | 301, 301, 301 |

The `read-profile.tsv` result was byte-identical across all repetitions: `66f32cc8526486ab5096ac77f6699316db231233220705a397ab14638fd07a89`.

Evidence: Slurm job `3459984`, repository `0250599effd8ddcd48870d3ae717589ee71756f3`, image `ba6d5e5acd51137d94fee735ae5a5ffcde3f1841d675ef5bc9c131bb673b47c4`.

### seqkit

| Metric | Measured repetitions |
|---|---|
| `file_count` | 30, 30, 30 |
| `read_count` | 75175997, 75175997, 75175997 |
| `base_count` | 4584222506, 4584222506, 4584222506 |
| `minimum_read_length` | 15, 15, 15 |
| `maximum_read_length` | 301, 301, 301 |

The `read-profile.tsv` result was byte-identical across all repetitions: `66f32cc8526486ab5096ac77f6699316db231233220705a397ab14638fd07a89`.

Evidence: Slurm job `3459985`, repository `0250599effd8ddcd48870d3ae717589ee71756f3`, image `4646ad26f8152674cda466d229ba0a5a495b9176ee4f3fc77f4d50cdf7e9d3d5`.

### seqkit_stats

| Metric | Measured repetitions |
|---|---|
| `file_count` | 30, 30, 30 |
| `read_count` | 75175997, 75175997, 75175997 |
| `base_count` | 4584222506, 4584222506, 4584222506 |
| `minimum_read_length` | 15, 15, 15 |
| `maximum_read_length` | 301, 301, 301 |

The `read-profile.tsv` result was byte-identical across all repetitions: `66f32cc8526486ab5096ac77f6699316db231233220705a397ab14638fd07a89`.

Evidence: Slurm job `3459986`, repository `0250599effd8ddcd48870d3ae717589ee71756f3`, image `315f69cd42259f5ef810ff0e9ea5ea6faa28ba8e2c3b6cc65b338d5ac2f4721c`.

## Cross-tool agreement

Agreement status: **within tolerance**.

| Metric | Tool means | Observed range | Allowed range | Status |
|---|---|---:|---:|---|
| `file_count` | `seqfu`=30, `seqkit`=30, `seqkit_stats`=30 | 0 | 0 | within |
| `read_count` | `seqfu`=75175997, `seqkit`=75175997, `seqkit_stats`=75175997 | 0 | 0 | within |
| `base_count` | `seqfu`=4584222506, `seqkit`=4584222506, `seqkit_stats`=4584222506 | 0 | 0 | within |
| `minimum_read_length` | `seqfu`=15, `seqkit`=15, `seqkit_stats`=15 | 0 | 0 | within |
| `maximum_read_length` | `seqfu`=301, `seqkit`=301, `seqkit_stats`=301 | 0 | 0 | within |

## Limitations

- The cohort is curated for workload diversity and is not population-representative.
- Minimum and maximum read lengths are cross-tool consistency observations because the admission catalog records reads and bases but not length extrema.
- Warm-cache repetitions characterize stable parsing after one warmup and do not measure cold shared-filesystem startup.
- The campaign covers the admitted gzip-compressed Illumina FASTQ files and does not generalize to every valid FASTQ encoding or compression format.

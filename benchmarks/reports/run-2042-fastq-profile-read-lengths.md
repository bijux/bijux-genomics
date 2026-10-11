<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: benchmarks/benchmark_evidence.py -->
# fastq.profile_read_lengths comparative

**Verdict: passed.** Exact row agreement with an independent FASTQ decoder establishes record coverage and read-length correctness for this cohort. It does not assess read transformation, biological representativeness, or every valid FASTQ encoding.

This is comparative evidence governed by the declared scientific profile.

## Scientific scope

Do both exact read-length backends recover every independently decoded per-record length across the complete real human short-read diversity cohort, and what resources does each implementation consume?

Comparison scope: thirty FASTQ files from twenty ancient and modern human runs spanning single- and paired-end libraries.

Accuracy claim: `exact_per_record_length_concordance_with_independent_fastq_decode`.

## Design

- Campaign: `run-2042`
- Evidence tier: `comparative`
- Scientific profile: `fastq-observational-qc`
- Tool closure: `seqfu`, `seqkit`
- Execution: apptainer under slurm, container-only
- Threads by tool: `seqfu`=1, `seqkit`=4
- Cache policy: warm
- Repetitions: 1 warmup and 3 measured
- Corpus: `human-short-read-diversity`
- Corpus portfolio: `human-short-read-diversity`
- Reference: none
- Databases: none
- Repeated result artifact: `read-lengths.json`

## Replay and comparison interface

- Repository commit: `7c604e7c596ba446911de3a167840e97c8acbffc`
- Immutable input closure: `491ffd8e8df3857616d1d401d1a4ab729a58d5a9e4c18a0951f308d1bfb966ad`
- Effective parameter closure: `31cd719ef9b42cab2eb2fb36183ccc1cc18bc3d0ca9617e8465d29dd4132dd3d`
- Accepted inputs: `bijux.fastq.artifact.single_end.v1`, `bijux.fastq.artifact.paired_end.v1`
- Normalized output: `bijux.benchmark.fastq.profile_read_lengths.bundle.v1`
- Output schema: `benchmarks/contracts/schemas/fastq-profile-read-lengths-result.schema.json`
- Tool-independent consumer: `benchmark.fastq.profile_read_lengths.compare`
- Substitution scope: normalized benchmark comparison only; this run does not claim operational downstream pipeline substitution

## Performance

| Tool | Wall mean (range), s | CPU mean, s | CPU use, % | Peak RSS mean (range), MiB | Throughput, MiB/s | Filesystem input/output operations | Output mean, bytes |
|---|---:|---:|---:|---:|---:|---:|---:|
| `seqfu` | 597.314 (593.998–599.259) | 593.035 | 99.3 | 44.6 (43.6–46.2) | 5.81 | 0.0/0.0 | 1592964957.0 |
| `seqkit` | 118.635 (118.320–118.966) | 151.397 | 127.6 | 56.4 (56.3–56.7) | 29.25 | 56.0/0.0 | 1592964958.0 |

## Scientific observations

### seqfu

| Metric | Measured repetitions |
|---|---|
| `file_count` | 30, 30, 30 |
| `read_count` | 75175997, 75175997, 75175997 |
| `minimum_read_length` | 15, 15, 15 |
| `mean_read_length` | 60.9798697581623, 60.9798697581623, 60.9798697581623 |
| `median_read_length` | 36.0, 36.0, 36.0 |
| `maximum_read_length` | 301, 301, 301 |
| `distinct_read_lengths` | 287, 287, 287 |
| `histogram_sha256` | 054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5, 054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5, 054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5 |

The `read-lengths.json` result was stable across repetitions by canonical bundle content with repetition-local histogram paths normalized: `3bd8e09540754442bd0ac0cb8f8c79c54e66da519e2cc1c9fbd85d20ca24ad8c`.

Evidence: Slurm job `3460317`, repository `7c604e7c596ba446911de3a167840e97c8acbffc`, image `ba6d5e5acd51137d94fee735ae5a5ffcde3f1841d675ef5bc9c131bb673b47c4`.

### seqkit

| Metric | Measured repetitions |
|---|---|
| `file_count` | 30, 30, 30 |
| `read_count` | 75175997, 75175997, 75175997 |
| `minimum_read_length` | 15, 15, 15 |
| `mean_read_length` | 60.9798697581623, 60.9798697581623, 60.9798697581623 |
| `median_read_length` | 36.0, 36.0, 36.0 |
| `maximum_read_length` | 301, 301, 301 |
| `distinct_read_lengths` | 287, 287, 287 |
| `histogram_sha256` | 054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5, 054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5, 054520be2dcdbeb3f6792060198baf86960c8c6f158d9f80443c956af94a8ec5 |

The `read-lengths.json` result was stable across repetitions by canonical bundle content with repetition-local histogram paths normalized: `57d23e6d37afd82d77792258b7c4403a849385ae8b1cda1fae4abb49e6f6ab91`.

Evidence: Slurm job `3460318`, repository `7c604e7c596ba446911de3a167840e97c8acbffc`, image `4646ad26f8152674cda466d229ba0a5a495b9176ee4f3fc77f4d50cdf7e9d3d5`.

## Cross-tool agreement

Agreement status: **within tolerance**.

| Metric | Tool means | Observed range | Allowed range | Status |
|---|---|---:|---:|---|
| `file_count` | `seqfu`=30, `seqkit`=30 | 0 | 0 | within |
| `read_count` | `seqfu`=75175997, `seqkit`=75175997 | 0 | 0 | within |
| `minimum_read_length` | `seqfu`=15, `seqkit`=15 | 0 | 0 | within |
| `mean_read_length` | `seqfu`=60.9798697582, `seqkit`=60.9798697582 | 0 | 0 | within |
| `median_read_length` | `seqfu`=36, `seqkit`=36 | 0 | 0 | within |
| `maximum_read_length` | `seqfu`=301, `seqkit`=301 | 0 | 0 | within |
| `distinct_read_lengths` | `seqfu`=287, `seqkit`=287 | 0 | 0 | within |

## Limitations

- The cohort is curated for workload diversity and is not population-representative.
- The independent truth surface validates lengths and record closure but not sequence identity beyond the preserved record identifiers.
- Warm-cache repetitions characterize stable parsing after one warmup and do not measure cold shared-filesystem startup.
- SeqFu tabulate is single-stream; its one-CPU allocation is intentional and recorded rather than falsely projecting the shared thread override into unsupported backend flags.

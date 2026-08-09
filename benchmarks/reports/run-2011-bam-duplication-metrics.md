<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::bam_observation::report -->
# bam.duplication_metrics qualification benchmark

**Verdict: passed.** This campaign compares duplicate-read accounting after each tool performs duplicate detection under one normalized count contract. Agreement is an interoperability check for this BAM, not an independent truth estimate of molecular library complexity.

## Scientific Scope

Do Picard and Samtools derive consistent duplicate-read counts for the same governed ancient horse alignment, and what resources does each implementation consume?

Comparison scope: duplicate detection over one governed coordinate-sorted EquCab3 BAM.

Accuracy claim: `cross_tool_duplicate_accounting_consistency_not_library_complexity_truth`.

## Reproducible Design

- Campaign: `run-2011`
- Repository commit: `27d88388e2fed29f96478c32ec5a5e8c73a7747d`
- Container catalog: `26eb025bbbdfe8a60e4f15f0832fb911605064ce1418d57c81d61a94bb66594b`
- Tool closure: `picard`, `samtools`
- Repetitions: 1 warmup and 3 measured
- Corpus: `ancient-horse-alignment-qualification`
- Result contract: `duplication-summary.tsv`

## Comparison Closure

- Immutable inputs: `e2c4b15564206e9de17af5678d56a8ad298fdb70a56a1f29f20878cc846fbd80`
- Effective parameters: `f88060f9c4c490ecc3ffd737e226974afefddcbd3220aed4dc46158b0d6d5991`
- Repository source: `27d88388e2fed29f96478c32ec5a5e8c73a7747d`

## Performance

| Tool | Slurm job | Wall mean (range), s | CPU mean, s | Peak RSS mean, MiB | Input bytes | Output mean, bytes |
|---|---:|---:|---:|---:|---:|---:|
| `picard` | `3473536` | 62.600 (60.030-67.280) | 24.280 | 8153.3 | 26592222 | 2282.0 |
| `samtools` | `3473537` | 1.447 (1.380-1.560) | 2.503 | 98.0 | 26592222 | 1044.0 |

## Scientific Observations

### picard

| Metric | Measured repetitions |
|---|---|
| `duplicate_reads` | 0.000000000000, 0.000000000000, 0.000000000000 |
| `duplication_fraction` | 0.000000000000, 0.000000000000, 0.000000000000 |
| `examined_reads` | 235311.000000000000, 235311.000000000000, 235311.000000000000 |

The `duplication-summary.tsv` artifact was identical across measured repetitions by normalized duplication-summary TSV bytes: `8a98808b503e8998b920eac1da06490516ad4a7c03d938f5ea36e65e2e1e82d5`.

Evidence: Slurm job `3473536`, node `cn135`, image `5126331cdaf67eef217e5bee96292a33bf1f80b0922fa2a980e3c8e621471bc1`, acceptance `bf8aa7e9e62bd92166acf2d637699ad54725fda1f9d9aa8a8ea7ff9c1b473e52`.

### samtools

| Metric | Measured repetitions |
|---|---|
| `duplicate_reads` | 0.000000000000, 0.000000000000, 0.000000000000 |
| `duplication_fraction` | 0.000000000000, 0.000000000000, 0.000000000000 |
| `examined_reads` | 235311.000000000000, 235311.000000000000, 235311.000000000000 |

The `duplication-summary.tsv` artifact was identical across measured repetitions by normalized duplication-summary TSV bytes: `8a98808b503e8998b920eac1da06490516ad4a7c03d938f5ea36e65e2e1e82d5`.

Evidence: Slurm job `3473537`, node `cn135`, image `21252c2a23f5464381f5602a60eef97ee03394e4376b9c6ab5b643974a949673`, acceptance `32f9804a3d445f3cb92b8a2150c148d5ac9719cc44d1050082249b55068e3c3a`.

## Cross-tool Agreement

Agreement status: **within tolerance**.

| Metric | Tool means | Observed range | Allowed range | Status |
|---|---|---:|---:|---|
| `examined_reads` | `picard`=235311.000000000000, `samtools`=235311.000000000000 | 0.000000000000 | 0.000000000000 | within |
| `duplicate_reads` | `picard`=0.000000000000, `samtools`=0.000000000000 | 0.000000000000 | 2.000000000000 | within |
| `duplication_fraction` | `picard`=0.000000000000, `samtools`=0.000000000000 | 0.000000000000 | 0.000001000000 | within |

## Limitations

- The corpus is one low-coverage ancient horse library and cannot establish agreement across all library structures or sequencing technologies.
- Picard and Samtools implement different duplicate-scoring and optical-duplicate models, so small count differences are interpreted within declared tolerances rather than as truth error.
- Duplicate fraction is an operational BAM statistic and does not independently estimate original molecular diversity.

<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: benchmarks/benchmark_evidence.py -->
# bam.genotyping qualification

**Verdict: passed.** This execution establishes reproducible candidate-site genotyping and resource use for the retained tool set. It does not establish genotype accuracy because no sample-level truth genotype is available.

This is qualification evidence. It proves the declared execution and metric contract on the named data, but it is not a comparative scientific benchmark and must not be used to rank tools.

## Scientific scope

How reliably and efficiently does each retained BAM genotyping tool recover independently catalogued EquCab3 candidate sites from a real low-coverage ancient horse alignment?

Comparison scope: candidate-site genotyping on one governed 10 Mb EquCab3 interval.

Accuracy claim: `candidate_site_recovery_not_genotype_truth`.

## Design

- Campaign: `run-2001`
- Evidence tier: `qualification`
- Scientific profile: `bam-biological-inference`
- Tool closure: `bcftools`
- Execution: apptainer under slurm, container-only
- Threads by tool: `bcftools`=4
- Cache policy: warm
- Repetitions: 1 warmup and 3 measured
- Corpus: `ena-prjeb22390-err2112767-bam`
- Corpus portfolio: `ancient-horse-alignment-qualification`
- Reference: `equcab3-gcf-002863925.1`
- Databases: `ensembl-113-equcab3-refseq-known-sites-10mb`, `ensembl-113-equcab3-refseq-genotyping-inputs-10mb`
- Repeated result artifact: `genotypes.tsv`

## Performance

| Tool | Wall mean (range), s | CPU mean, s | Peak RSS mean (range), MiB | Output mean, bytes |
|---|---:|---:|---:|---:|
| `bcftools` | 7.750 (7.720–7.770) | 8.013 | 934.5 (933.0–936.2) | 577469.3 |

## Scientific observations

### bcftools

| Metric | Measured repetitions |
|---|---|
| `total_records` | 139907, 139907, 139907 |
| `covered_sites` | 21, 21, 21 |
| `called_genotypes` | 21, 21, 21 |
| `alternate_genotypes` | 3, 3, 3 |
| `mean_depth_at_covered_sites` | 1.0, 1.0, 1.0 |

The `genotypes.tsv` result was byte-identical across all repetitions: `0f6907c7b606b91ad4ddf2cac410577f7a38aadfc2be16a5d88c0a285f234761`.

Evidence: Slurm job `3459734`, repository `1e1c136988a2932b77537786fdc759a193149149`, image `408eeb428a48286e9f8160571db10e4cb1b519fea336983a41e7bcab4551e761`.

## Cross-tool agreement

This campaign has one tool, so cross-tool agreement is not applicable.

## Limitations

- The Ensembl panel is independent candidate-site evidence but is not a sample-level genotype truth set.
- The first 10 Mb of chromosome 1 is a qualification interval, not a genome-wide accuracy estimate.
- Warm-cache repetitions measure stable tool execution after one unmeasured warmup; they do not estimate cold filesystem startup cost.

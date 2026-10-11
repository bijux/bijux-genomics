<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: bijux-dna-bench::workflow::bam_observation::report -->
# bam.genotyping qualification benchmark

**Verdict: passed.** This execution establishes reproducible candidate-site genotyping and resource use for the retained tool set. It does not establish genotype accuracy because no sample-level truth genotype is available.

## Scientific Scope

How reliably and efficiently does each retained BAM genotyping tool recover independently catalogued EquCab3 candidate sites from a real low-coverage ancient horse alignment?

Comparison scope: candidate-site genotyping on one governed 10 Mb EquCab3 interval.

Accuracy claim: `candidate_site_recovery_not_genotype_truth`.

## Reproducible Design

- Campaign: `run-2001`
- Repository commit: `9348a2417384417ab59052193b56717b69cb0309`
- Container catalog: `d6b352cec629b12396c93e67968a42b43f576550038cda25d16cc88db2639250`
- Tool closure: `bcftools`
- Repetitions: 1 warmup and 3 measured
- Corpus: `ancient-horse-alignment-qualification`
- Result contract: `genotypes.tsv`

## Comparison Closure

- Immutable inputs: `1a3b9a88c4186e1b32e686fb1eb234363c9c56f785a5c2c13fee29ec455ef106`
- Effective parameters: `252bb6b96ad335861ae1a07188a1d823f5a8cd36fbabbdda14551ed8eb00912d`
- Repository source: `9348a2417384417ab59052193b56717b69cb0309`

## Performance

| Tool | Slurm job | Wall mean (range), s | CPU mean, s | Peak RSS mean, MiB | Input bytes | Output mean, bytes |
|---|---:|---:|---:|---:|---:|---:|
| `bcftools` | `3463105` | 8.193 (8.140-8.220) | 8.693 | 930.0 | 2574295332 | 4484572.0 |

## Scientific Observations

### bcftools

| Metric | Measured repetitions |
|---|---|
| `alternate_genotypes` | 3.000000000000, 3.000000000000, 3.000000000000 |
| `called_genotypes` | 21.000000000000, 21.000000000000, 21.000000000000 |
| `covered_sites` | 21.000000000000, 21.000000000000, 21.000000000000 |
| `mean_depth_at_covered_sites` | 1.000000000000, 1.000000000000, 1.000000000000 |
| `total_records` | 139907.000000000000, 139907.000000000000, 139907.000000000000 |

The `genotypes.tsv` artifact was identical across measured repetitions by normalized genotype TSV bytes: `0f6907c7b606b91ad4ddf2cac410577f7a38aadfc2be16a5d88c0a285f234761`.

Evidence: Slurm job `3463105`, node `cn162`, image `b019a573e85877eb72be9a3285ce4c28701828f0cadb146fdf58d77b40394c63`, acceptance `fab86e8447b4f45fea41e6ab5e899266c9520c943b486e17c2f7f099612ebf03`.

## Cross-tool Agreement

Agreement status: **not applicable**.

| Metric | Tool means | Observed range | Allowed range | Status |
|---|---|---:|---:|---|

## Limitations

- The Ensembl panel is independent candidate-site evidence but is not a sample-level genotype truth set.
- The first 10 Mb of chromosome 1 is a qualification interval, not a genome-wide accuracy estimate.
- Warm-cache repetitions measure stable tool execution after one unmeasured warmup; they do not estimate cold filesystem startup cost.

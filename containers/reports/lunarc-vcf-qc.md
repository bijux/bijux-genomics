# LUNARC VCF quality-control validation

This report records real PLINK 2.00a6 execution on a public 100-sample,
chromosome 22 panel from 1000 Genomes Phase 3. The run used an immutable
Apptainer image, checksum-pinned input, clean commit-addressed checkout,
configuration-driven LUNARC policy, explicit invalid-input check, and terminal
Slurm accounting.

The machine-readable identities are in the
[repository evidence index](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/lunarc-vcf-qc.json).
Raw manifests, logs, outputs, and accounting remain below the governed LUNARC
study root.

## Passing execution

| Stage | Tool | Run | Slurm job | Elapsed | Principal output |
|---|---|---:|---:|---:|---|
| regional allele frequencies | PLINK 2.00a6 | `run-1063` | `3457946` | 6s | 651,851-byte frequency table for 41,924 records and 100 samples |

Version, help, real-input, invalid-input, image, input, output, repository, and
Slurm accounting contracts all passed. The negative check returned PLINK's
documented nonzero input error instead of succeeding or fabricating output.

## Claim boundary

This run proves the PLINK 2 runtime and regional allele-frequency stage on
non-fixture data. It does not establish Hardy-Weinberg thresholds, missingness
policy, ancestry inference, principal-component quality, kinship accuracy,
relatedness thresholds, imputation quality, repeated performance variance,
security acceptance, or release qualification. Those claims require separate
predeclared analyses and evidence; this stage result must not be used as a
population-genetic interpretation.

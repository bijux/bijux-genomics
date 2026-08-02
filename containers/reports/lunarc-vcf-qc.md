# LUNARC VCF quality-control validation

This report records real PLINK 2.00a6 and PLINK 1.90b7.2 execution on a public
100-sample, chromosome 22 panel from 1000 Genomes Phase 3. The runs used
immutable Apptainer images, checksum-pinned inputs, clean commit-addressed
checkouts, configuration-driven LUNARC policy, explicit invalid-input checks,
and terminal Slurm accounting.

The machine-readable identities are in the
[repository evidence index](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/lunarc-vcf-qc.json).
Raw manifests, logs, outputs, and accounting remain below the governed LUNARC
study root.

## Passing execution

| Stage | Tool | Run | Slurm job | Elapsed | Principal output |
|---|---|---:|---:|---:|---|
| regional allele frequencies | PLINK 2.00a6 | `run-1063` | `3457946` | 6s | 651,851-byte frequency table for 41,924 records and 100 samples |
| PED/MAP interoperability export | PLINK 2.00a6 | `run-1064` | `3457961` | 24s | 16,771,500-byte PED and 670,784-byte MAP used by the GERMLINE qualification |
| regional allele frequencies | PLINK 1.90b7.2 | `run-1066` | `3457971` | 7s | 1,760,850-byte frequency table |

Version, help, real-input, invalid-input, image, input, output, repository, and
Slurm accounting contracts all passed. The negative check returned PLINK's
documented nonzero input error instead of succeeding or fabricating output.

## Claim boundary

These runs prove the PLINK 1 and PLINK 2 runtimes, regional allele-frequency
execution, and PED/MAP interchange on non-fixture data. They do not establish
Hardy-Weinberg thresholds, missingness policy, ancestry inference,
principal-component quality, kinship accuracy, relatedness thresholds,
imputation quality, repeated performance variance, security acceptance, or
release qualification. Those claims require separate predeclared analyses and
evidence; these stage results must not be used as population-genetic
interpretations.

# LUNARC BAM coverage validation

This report records BEDTools 2.31.1 running `genomecov` against the public
horse whole-genome sample `ERR2112767`. The run used the 23,312,078-byte sorted
BAM already admitted to the governed LUNARC corpus, an immutable Apptainer
image, a clean commit-addressed checkout, runtime network isolation, and
terminal Slurm accounting.

The machine-readable identities are in the
[repository evidence index](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/lunarc-bam-coverage.json).
Raw inputs, images, logs, and outputs remain under the governed LUNARC study
root.

## Execution evidence

| Stage | Tool | Run | Slurm job | Elapsed | Principal output |
|---|---|---:|---:|---:|---|
| BAM coverage distribution | BEDTools 2.31.1 | `run-1074` | `3458053` | 1m17s | 234,720-byte genome coverage table |

The qualification executed version, help, real-sample, and invalid-input
checks. The real command completed successfully, while the deliberately
missing BAM was rejected. The image build also preserved the Git object
identity of its repository-owned wrapper in build evidence.

## Claim boundary

This establishes the BEDTools container interface and a real `bam.coverage`
stage execution. It does not establish a minimum sequencing depth, breadth of
coverage suitable for a downstream analysis, agreement with another coverage
implementation, benchmark variance, security acceptance, or release
qualification. Those claims require predeclared thresholds and additional
evidence dimensions.

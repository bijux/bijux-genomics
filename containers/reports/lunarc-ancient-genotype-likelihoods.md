# LUNARC ancient-DNA genotype-likelihood validation

This report records runtime and stage qualification of ANGSD 0.941 on public
ancient-horse run `ERR2112767` (`Borly4_PAVH8_4978`, BioSample
`SAMEA104233654`). The checksum-admitted coordinate-sorted BAM and BAI were
processed without runtime network access by an immutable Apptainer image built
from exact ANGSD, HTSlib, and HTScodecs source archives.

The machine-readable identities are in the
[repository evidence index](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/lunarc-ancient-genotype-likelihoods.json).
Raw inputs, image, logs, and outputs remain under the LUNARC study root read
from `containers/contracts/sites/lunarc.yml` and archived with the run.

## Execution

| Purpose | Tool | Run | Slurm job | Status | Elapsed | Principal result |
|---|---|---:|---:|---|---:|---|
| estimate one-sample genotype likelihoods and allele frequencies | ANGSD 0.941 | `run-1094` | `3458115` | passed | 2s | Beagle likelihoods and MAF estimates for 388,677 retained sites |

The command used the Samtools genotype-likelihood model (`-GL 1`), Beagle
likelihood output (`-doGlf 2`), major/minor inference, and allele-frequency
estimation. Mapping and base qualities below 20 were excluded. The run read
404,955 sites and retained 388,677; both compressed outputs contain a header
plus 388,677 data rows. Version, help, real execution, negative-option
rejection, input hashes, image hash, repository commit, and terminal Slurm
accounting passed in one governed run.

## Claim boundary

This evidence closes ANGSD's container runtime and the declared
`bam.genotyping` execution surface for one public ancient sample. It proves
that the pinned implementation can read the admitted BAM/BAI pair and emit
non-empty genotype-likelihood and allele-frequency products under the
governed LUNARC contract.

It does not establish genotype accuracy, contamination correction, damage
robustness, appropriate filtering thresholds, population suitability,
cross-tool agreement, benchmark variance, or release qualification. Those
claims require predeclared controls, references, truth data, comparison
methods, and scientific acceptance criteria.

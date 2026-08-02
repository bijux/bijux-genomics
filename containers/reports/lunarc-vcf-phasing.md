# LUNARC VCF phasing validation

This report records real stage execution for BCFtools 1.20, SHAPEIT5 5.1.1,
and Eagle 2.4.1 on a public 2,504-sample 1000 Genomes Phase 3 chromosome 22
cohort. Every job ran through an immutable Apptainer image and a manifest-bound
checkout of repository commit `c07932e6374fc0ef1511a6341223fd8e0baf226c`.

The machine-readable identities are in
[`lunarc-vcf-phasing.json`](lunarc-vcf-phasing.json). Raw manifests, logs,
outputs, and Slurm records remain below the governed LUNARC study root and are
verified by the SHA-256 values in that index.

## What passed

| Stage | Tool | Run | Slurm job | Elapsed | Result |
|---|---|---:|---:|---:|---|
| common-variant phasing | SHAPEIT5 5.1.1 | `run-1033` | `3457723` | 4m42s | 5,004,945-byte phased BCF |
| multiallelic normalization | BCFtools 1.20 | `run-1035` | `3457731` | 6s | 8,009,257-byte indexed VCF |
| common-variant phasing | Eagle 2.4.1 | `run-1036` | `3457739` | 2m59s | 8,004,844-byte phased VCF |

Each run also passed its version, help, and invalid-input contracts. The input,
image, output, manifest, and accounting identities are immutable evidence, not
fixture assertions.

## Why normalization is explicit

Eagle run `run-1034` rejected the unmodified panel because it contained
multiallelic records. That failed result remains in the evidence tree. BCFtools
run `run-1035` selected `22:20000000-22000000`, split multiallelic records with
`norm -m -any`, indexed the derived VCF, and recorded its parent and output
checksums. Eagle run `run-1036` consumed the admitted derived corpus rather
than modifying the raw panel or hiding the incompatibility.

## Claim boundary

These runs prove image execution, command interfaces, failure behavior, real
input handling, governed derivation, and output production on LUNARC. They do
not yet establish switch-error accuracy, truth-set concordance, comparative
quality, repeated benchmark variance, vulnerability acceptance, or release
qualification. Both phasers therefore remain `unverified` in the canonical
catalog until those independent evidence dimensions pass.

The source cohort is already phased. A later scientific campaign must mask or
unphase a predefined truth subset, declare metrics before execution, and
compare outputs against the held-out truth without using these stage results
as accuracy evidence.

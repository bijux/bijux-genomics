# LUNARC VCF relatedness validation

This report records a real IBDseq execution and the discovered IBDNe input
boundary on a public 100-sample 1000 Genomes chromosome 22 slice. The source
panel contains 44,115 biallelic records from `22:20000000-22000000`.

The machine-readable identities are in
[`lunarc-vcf-relatedness.json`](lunarc-vcf-relatedness.json). Raw manifests,
logs, outputs, and Slurm accounting remain under the governed LUNARC study
root.

## Passing execution

BCFtools run `run-1038` created and indexed the deterministic 100-sample panel
in 11 seconds. IBDseq r1206 run `run-1039` processed that panel in 3 seconds
and emitted 124 segment rows plus its log and LD-filter record. Version, help,
invalid-input, image, input, and output contracts passed.

## IBDNe boundary

IBDNe 23Apr20.ae9 itself starts and validates its command interface, but no
successful demographic run is claimed:

- `run-1040` demonstrated that bootstrap resampling is invalid with only one
  chromosome.
- `run-1041` demonstrated that the default cross-chromosome sample filter is
  also inapplicable to this validation slice.
- `run-1042` disabled both settings and exposed the fundamental incompatibility:
  IBDseq reports LOD in its final segment column, whereas IBDNe expects the
  hap-IBD output convention with segment length in centimorgans.

The failed evidence is retained. No further parameter relaxation is valid.
The next run requires an authenticated hap-IBD container and a governed,
population-aware whole-genome cohort. Until those prerequisites exist, IBDNe
remains runtime-visible but stage-unverified.

## Claim boundary

The IBDseq result proves real stage execution, not relatedness accuracy. The
short, header-order cohort is unsuitable for demographic inference, population
comparison, false-positive calibration, or benchmark claims. It must not be
used to label IBDNe scientifically qualified.

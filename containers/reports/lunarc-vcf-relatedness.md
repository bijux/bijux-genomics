# LUNARC VCF relatedness validation

This report records real IBDseq and IBDNe executions on public 100-sample
1000 Genomes chromosome 22 data. The initial diagnostic panel contains 44,115
biallelic records from `22:20000000-22000000`; the qualifying chromosome-wide
panel contains 1,055,454 biallelic SNP records.

The machine-readable identities are in the
[repository evidence index](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/lunarc-vcf-relatedness.json).
Raw manifests, logs, outputs, and Slurm accounting remain under the governed
LUNARC study root.

## Passing execution

BCFtools run `run-1051` created and indexed the deterministic chromosome-wide
100-sample panel in 51 seconds. IBDseq r1206 run `run-1052` processed 1,055,454
records in 27 seconds and emitted 3,282 segment rows. IBDNe run `run-1053`
read every segment, retained 131 segments at its documented 2 cM threshold,
and emitted a 301-generation effective-population-size trajectory in 3
seconds. Version, help, invalid-input, image, input, output, and Slurm
accounting contracts passed for all three runs.

The initial `run-1038` and `run-1039` pair remains useful diagnostic evidence:
the 2 Mb panel completed in 11 and 3 seconds respectively, with 124 IBDseq
segment rows, but it was not broad enough for IBDNe's default threshold.

## IBDNe diagnostic boundary

The retained failed runs explain why the narrow panel could not qualify IBDNe
23Apr20.ae9:

- `run-1040` demonstrated that bootstrap resampling is invalid with only one
  chromosome.
- `run-1041` disabled bootstrap resampling, but no segments remained after the
  2 cM minimum and default sample-pair filter.
- `run-1042` disabled both settings and confirmed that no segment from the
  narrow interval survives IBDNe's scientifically meaningful 2 cM minimum.
  Inspection of the checksum-pinned IBDNe source confirmed that its parser
  accepts IBDseq's eight-column records and derives segment lengths from the
  required PLINK genetic map; the final IBDseq field is treated as a score,
  not a length.

The failed evidence is retained. No further parameter relaxation is valid.
Run `run-1053` resolves the runtime and stage boundary without relaxing the
threshold. A population-aware, multi-chromosome cohort is still required
before interpreting the trajectory scientifically or enabling chromosome
bootstrap confidence intervals.

## Claim boundary

These results prove real stage execution, not relatedness accuracy or a human
population-history estimate. The cohort follows source header order, covers
one chromosome, and disables bootstrap resampling. It is unsuitable for
population comparison, confidence intervals, false-positive calibration, or
scientific benchmark claims. The emitted trajectory must not be interpreted as
a demographic result or used to label IBDNe scientifically qualified.

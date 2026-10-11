# LUNARC VCF relatedness validation

This report records real IBDseq, IBDNe, MORGAN IBDHap, and GERMLINE executions
on public 100-sample 1000 Genomes chromosome 22 data. The initial diagnostic
panel contains 44,115 biallelic records from `22:20000000-22000000`; the
qualifying chromosome-wide panel contains 1,055,454 biallelic SNP records.

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

MORGAN IBDHap 3.4 run `run-1067` evaluated HG00096 and HG00097 over the same
2 Mb interval in 7 seconds. Its adapter retained 41,705 eligible phased loci,
recorded the exclusion of 219 loci with uninformative allele frequencies, and
selected 500 evenly spaced markers for the bounded analysis. It emitted 500
rows of 15-state posterior four-gamete probabilities whose sums remained
within 0.0002 of one.

GERMLINE 1.5.3 run `run-1069` evaluated the complete 100-sample PED/MAP export
in 3 seconds and emitted 18 candidate segment rows. Its governed adapter
rejected absent PED/MAP inputs with a nonzero exit before invoking an upstream
binary that otherwise reports success for that invalid condition. The
superseded `run-1068` execution produced the same outputs but is retained only
as provenance because its manifest mislabeled the cohort as a sample pair.

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
population-history estimate. IBDHap emits posterior state probabilities, not
called segments. GERMLINE emits candidate segments, not validated kinship or
accuracy. The cohort follows source header order, covers one chromosome, and
disables IBDNe bootstrap resampling. It is unsuitable for population
comparison, confidence intervals, false-positive calibration, or scientific
benchmark claims. Neither the IBDNe trajectory nor any relatedness output may
be interpreted scientifically without a predeclared population-aware cohort,
truth or comparison design, and method-specific acceptance thresholds.

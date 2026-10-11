# VCF IBD Stage

## Purpose
Define the governed relatedness-inference boundary for pairwise IBD segments without hiding the upstream phasing dependency or the downstream demography handoff.

## Scope
This science surface covers:
- `vcf.phasing` as the upstream haplotype-preparation boundary when an admitted IBD backend requires phased input.
- `vcf.ibd` as the planned pairwise segment-calling stage with an active normalized pairwise contract.
- `vcf.demography` only as a downstream consumer of IBD summaries, not as part of the segment-calling output itself.

## Non-goals
- Treating IBD outputs as final demography estimates without a demography stage.
- Pretending that unphased and phased IBD inputs are interchangeable.

## Contracts
- `vcf.ibd` emits `ibd_segments` with schema `bijux.vcf.ibd.v1`.
- Segment-calling backends are `germline` and `ibdseq`. MORGAN IBDHap is an IBD-state probability backend: it emits posterior probabilities over four-gamete states and must not be represented as a direct segment caller.
- Required metrics include `pair_count`, `rows`, `status`, `insufficient_reason`, and `insufficient_overlap_probe`.
- Every normalized pair row must keep `sample_a`, `sample_b`, `segment_count`, `total_length`, `overlap_marker_count`, and `status`.
- When `vcf.demography` is scheduled, its input contract must consume a filtered and version-pinned `vcf.ibd` output rather than re-deriving IBD implicitly.

## Validity Limits
- Segment calls are affected by `vcf.phasing` quality, genotype error, and marker density.
- Different tools and minimum-length settings can shift segment counts and lengths materially.
- Cross-tool parity is only meaningful when all backends emit the same normalized pair-row schema and the same phasing/panel assumptions are held fixed.
- IBDHap comparisons require the same genetic map, marker subset, 2011 state-transition model, population-kinship prior, kinship-change rate, transition null fraction, and genotyping-error rate. Its probability surface needs an explicit downstream calling rule before it can be compared with segment tables.

# OTU vs ASV Semantics

## Purpose
Describe OTU clustering versus ASV inference contracts for FASTQ ecology workflows.

## Scope
`fastq.otu_clustering` and `fastq.asv_inference` stage interpretation.

## Non-goals
- Declaring one method universally superior.
- Converting historical OTU studies into ASV outputs automatically.

## Contracts
- OTU and ASV outputs are treated as non-comparable result families.
- Identity threshold (OTU) and denoiser model assumptions (ASV) are explicit.
- Reports must state whether downstream metrics are OTU- or ASV-derived.
- Current policy: ASV is not implemented in pre-HPC baseline; OTU is the primary supported ecological quantification path.
- OTU outputs must use stable identifier generation and reproducible clustering parameters.

## Examples
- OTU at 97% identity for broad comparability.
- ASV for higher-resolution low-divergence communities.

## Failure modes
- Mixing OTU and ASV matrices in one analysis causes invalid comparisons.
- Under-powered depth destabilizes ASV inference.

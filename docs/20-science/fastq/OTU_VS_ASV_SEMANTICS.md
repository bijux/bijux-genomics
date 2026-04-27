# OTU vs ASV Semantics

## Purpose
Describe OTU clustering versus ASV inference contracts for FASTQ ecology workflows.

## Scope
`fastq.cluster_otus` and `fastq.infer_asvs` stage interpretation.

## Non-goals
- Declaring one method universally superior.
- Converting historical OTU studies into ASV outputs automatically.

## Contracts
- OTU-stage artifacts, parameters, and clustering assumptions live in
  [domain/fastq/stages/cluster_otus.yaml](../../../domain/fastq/stages/cluster_otus.yaml).
- ASV-stage artifacts, parameters, and denoising assumptions live in
  [domain/fastq/stages/infer_asvs.yaml](../../../domain/fastq/stages/infer_asvs.yaml).
- The pinned default backends for both stages live in
  [domain/fastq/docs/DEFAULT_SETTINGS.md](../../../domain/fastq/docs/DEFAULT_SETTINGS.md).
- Route-level comparability and amplicon-only admission boundaries live in
  [domain/fastq/route_policies.toml](../../../domain/fastq/route_policies.toml).

## Examples
- OTU at 97% identity for broad comparability.
- ASV for higher-resolution low-divergence communities.

## Failure modes
- Mixing OTU and ASV matrices in one analysis causes invalid comparisons.
- Under-powered depth destabilizes ASV inference.

# Primer Assumptions

## Purpose
Define assumptions for primer normalization in ecology-first FASTQ workflows.

## Scope
`fastq.normalize_primers` stage behavior and interpretation limits.

## Non-goals
- Defining primer sets for every marker gene.
- Replacing marker-specific wet-lab guidance.

## Contracts
- Primer-bank preparation and provenance live in
  [domain/fastq/stages/prepare_primer_bank.yaml](../../../domain/fastq/stages/prepare_primer_bank.yaml).
- Primer-normalization inputs, outputs, and pairing invariants live in
  [domain/fastq/stages/normalize_primers.yaml](../../../domain/fastq/stages/normalize_primers.yaml).
- Pinned mismatch, overlap, and orientation defaults live in
  [domain/fastq/docs/DEFAULT_SETTINGS.md](../../../domain/fastq/docs/DEFAULT_SETTINGS.md).

## Examples
- ITS pollen runs trim primer tails before chimera detection.
- 16S eDNA runs retain reads with one-sided matches and tag them.

## Failure modes
- Wrong primer bank inflates false negatives.
- Over-aggressive trimming biases abundance estimates.

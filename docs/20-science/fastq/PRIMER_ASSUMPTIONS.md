# Primer Assumptions

## Purpose
Define assumptions for primer normalization in ecology-first FASTQ workflows.

## Scope
`fastq.primer_normalization` stage behavior and interpretation limits.

## Non-goals
- Defining primer sets for every marker gene.
- Replacing marker-specific wet-lab guidance.

## Contracts
- Primer references must be versioned and traceable.
- Ambiguous primer matches must be reported, not silently dropped.
- Primer-trimmed outputs must preserve read pairing invariants.

## Examples
- ITS pollen runs trim primer tails before chimera detection.
- 16S eDNA runs retain reads with one-sided matches and tag them.

## Failure modes
- Wrong primer bank inflates false negatives.
- Over-aggressive trimming biases abundance estimates.

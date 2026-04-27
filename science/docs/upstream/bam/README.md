# BAM Upstream Closure

`science/docs/upstream/bam/` is the tracked closure-control surface for governed
BAM tools in `bijux-genomics`.

[../README.md](../README.md) defines the broader upstream archive boundary, and
[BAM_PRODUCTION_CLOSURE_LEDGER.tsv](BAM_PRODUCTION_CLOSURE_LEDGER.tsv) is the
conservative blocker ledger for BAM evidence and promotion review.

## Scope

- governed BAM tool identities and upstream locators
- citation closure debt for supported and planned BAM tools
- explicit release-review blockers for BAM-stage scientific promotion

## Layout

- [BAM_PRODUCTION_CLOSURE_LEDGER.tsv](BAM_PRODUCTION_CLOSURE_LEDGER.tsv)
  tracked blocker ledger for governed BAM tools

## Rules

- keep BAM closure debt separate from FASTQ closure debt
- do not mark a BAM tool closed from plausible citation prose alone
- keep placeholder, local-only, and unresolved upstream identities visible until
  the governed tool contract and evidence surface are repaired together

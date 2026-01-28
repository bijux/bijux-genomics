# FASTQ Iteration 1 (Definition of Done)

FASTQ iteration 1 is complete when:

- Trim/filter/stats are authoritative and stable.
- One golden path exists: validate → trim → merge → filter → stats.
- Delta metrics are computed and trusted for trim and filter.
- Tool selection is gated by image QA + tool QA + stage invariants.
- One PE real-data E2E run is reproducible.

Iteration 2 begins only after this checklist is met.

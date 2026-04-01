# bijux-dna-pipelines Test Taxonomy

Stable test entrypoints:
- `boundaries.rs` for architecture and guardrail coverage.
- `contracts.rs` for defaults, profiles, and registry contracts.
- `guardrails.rs` for crate-local guardrail smoke checks.
- `invariant_fast.rs` for fast FASTQ invariant coverage.

Intent directories:
- `boundaries/` for layout and boundary contracts.
- `contracts/` for defaults, profile, and registry behavior.
- `determinism/` for reproducibility notes and future coverage.
- `schemas/` for docs and public-surface lock coverage.

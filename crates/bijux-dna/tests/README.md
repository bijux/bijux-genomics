# bijux-dna Test Taxonomy

Intent buckets in this crate:

- `boundaries`: layering, ownership, dependency, and public-surface guardrails.
- `contracts`: CLI behavior, dry-run, bank, and HPC layout contracts.
- `determinism`: reproducibility and stable-output checks.
- `snapshots`: help output and public-surface stability locks.

Speed model:

- **fast**: unit/contract tests without large fixtures or external tool execution.
- **slow**: heavy integration/snapshot regeneration tests; run in slow gate only.

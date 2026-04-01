# bijux-dna-environment Test Taxonomy

Intent buckets in this crate:

- `boundaries/`: layering and ownership guardrails.
- `contracts/`: API, resolver, and catalog behavioral contracts.
- `determinism/`: reproducibility and stable-output checks.
- `schemas/`: schema and public-surface stability snapshots.

Speed model:

- **fast**: unit/contract tests without large fixtures or external tool execution.
- **slow**: heavy integration/snapshot regeneration tests; run in slow gate only.

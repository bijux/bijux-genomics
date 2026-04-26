# bijux-dna-domain-bam Test Taxonomy

Intent buckets in this crate:

- `boundaries`: layering and ownership guardrails.
- `contracts`: API/data/schema behavioral contracts.
- `determinism`: reproducibility and stable-output checks.
- `schemas`: schema/public-surface stability snapshots.

Speed model:

- **fast**: unit/contract tests without large fixtures or external tool execution.
- **slow**: heavy integration/snapshot regeneration tests; run in slow gate only.

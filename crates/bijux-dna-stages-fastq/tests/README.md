# bijux-dna-stages-fastq Test Taxonomy

Intent buckets in this crate:

- `boundaries`: layering, purity, and ownership guardrails.
- `contracts`: API/data/schema behavioral contracts.
- `determinism`: reproducibility and stable-output checks.
- `schemas`: schema/public-surface stability snapshots.
- `semantics`: report and metrics behavior checks.

Speed model:

- **fast**: unit/contract tests without large fixtures or external tool execution.
- **slow**: heavy integration/snapshot regeneration tests; run in slow gate only.

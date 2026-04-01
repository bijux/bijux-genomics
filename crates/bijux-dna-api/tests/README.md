# bijux-dna-api Test Taxonomy

Intent buckets in this crate:

- `boundaries/`: layering, ownership, and layout guardrails.
- `contracts/`: public behavior and integration contracts.
- `schemas/`: schema stability and public-surface snapshots.
- root integration files: crate-level harnesses and workspace support helpers.

Speed model:

- **fast**: unit/contract tests without large fixtures or external tool execution.
- **slow**: heavy integration/snapshot regeneration tests; run in slow gate only.

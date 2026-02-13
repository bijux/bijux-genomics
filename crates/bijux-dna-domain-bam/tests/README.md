# bijux-dna-domain-bam Test Taxonomy

Intent buckets in this crate:

- \: layering and ownership guardrails.
- \: API/data/schema behavioral contracts.
- \: reproducibility and stable-output checks.
- \: schema/public-surface stability snapshots.

Speed model:

- **fast**: unit/contract tests without large fixtures or external tool execution.
- **slow**: heavy integration/snapshot regeneration tests; run in slow gate only.

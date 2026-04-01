# bijux-dna-environment-qa Test Taxonomy

Intent buckets in this crate:

- `boundaries/`: layering and ownership guardrails.
- `contracts/`: artifact and workflow behavioral contracts.
- `determinism/`: reproducibility and stable-output checks.
- `support/`: shared fixtures used by contract tests.

Speed model:

- **fast**: unit/contract tests without large fixtures or external tool execution.
- **slow**: heavy integration/snapshot regeneration tests; run in slow gate only.

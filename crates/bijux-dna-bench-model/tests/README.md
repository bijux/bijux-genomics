# bijux-dna-bench-model Test Taxonomy

Intent buckets in this crate:

- `boundaries`: layering and ownership guardrails.
- `contracts`: suite validation and benchmark contract behavior.
- `determinism`: reproducibility and stable-output checks.
- `schemas`: schema and public-surface stability checks.
- `semantics`: explainability and metric semantics behavior.

Speed model:

- **fast**: unit/contract tests without large fixtures or external tool execution.
- **slow**: heavy integration/snapshot regeneration tests; run in slow gate only.

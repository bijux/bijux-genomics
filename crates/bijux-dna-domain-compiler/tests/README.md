# bijux-dna-domain-compiler Test Taxonomy

Intent buckets in this crate:

- `guardrails`: policy and ownership checks.
- `determinism_generated_outputs`: repeated compilation stability.
- `planned_tool_registry_boundaries`: planned tool exclusion and visibility contracts.
- `contracts`: reserved API/data/schema behavioral contracts.
- `boundaries`: reserved layering and ownership checks.
- `schemas`: reserved schema/public-surface stability checks.

Speed model:

- **fast**: unit/contract tests without large fixtures or external tool execution.
- **slow**: heavy integration/snapshot regeneration tests; run in slow gate only.

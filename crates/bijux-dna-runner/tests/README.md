# bijux-dna-runner Test Taxonomy

Intent buckets in this crate:

- `boundaries/`: layering, ownership, dependency, and backend guardrails.
- `contracts/`: runner-facing behavioral contracts.
- `determinism/`: reproducibility and stable-output checks.
- root integration files: focused crate-level checks that do not belong to a deeper namespace.

Speed model:

- **fast**: unit/contract tests without large fixtures or external tool execution.
- **slow**: heavy integration/snapshot regeneration tests; run in slow gate only.

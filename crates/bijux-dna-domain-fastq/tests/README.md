# bijux-dna-domain-fastq Test Taxonomy

Intent buckets in this crate:

- `tests/boundaries/`: layering, ownership, purity, and policy guardrails.
- `tests/contracts/`: API, domain manifest, schema, and stage contract behavior.
- `tests/determinism/`: reproducibility and stable-output checks.
- `tests/semantics/`: domain semantics for params, invariants, retention, and observers.

Speed model:

- **fast**: unit/contract tests without large fixtures or external tool execution.
- **slow**: heavy integration/snapshot regeneration tests; run in slow gate only.

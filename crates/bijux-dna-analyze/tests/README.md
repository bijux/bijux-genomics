# bijux-dna-analyze Test Taxonomy

Intent buckets in this crate:
- `boundaries`: architecture tree, ownership, and public-surface guardrails
- `contracts`: user-visible artifact, pipeline, facts, and API behavior contracts
- `determinism`: stable fixture and serialization behavior
- `schemas`: SQLite compatibility and schema evolution checks
- `semantics`: ranking, comparison, and decision-trace meaning

Supporting directories:
- `fixtures/`: durable test inputs
- `snapshots/`: blessed contract outputs

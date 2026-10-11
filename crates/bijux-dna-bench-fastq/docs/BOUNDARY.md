# bijux-dna-bench-fastq boundary

Owner: bijux-dna-bench-fastq
Scope: Synthetic FASTQ parser probes and explicit-input readiness coverage.
Allowed inputs: Governed stage plans, read/reference inputs, serving snapshots and scope records.
Forbidden dependencies: Frontend crates bijux-dna and bijux-dna-dev; tests must not become production dependencies.
Forbidden effects: No planner acquisition, backend execution or process-global configuration.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-fastq --all-targets

[Architecture](ARCHITECTURE.md) defines the internal ownership and effect boundaries. [Public API](PUBLIC_API.md) records supported inputs and outputs. [Tests](TESTS.md) describes feature-specific qualification and evidence limits. Dependency declarations and the workspace boundary graph are executable constraints; this document does not establish new scientific or hosted performance claims.

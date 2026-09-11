# bijux-dna-bench-vcf-evidence boundary

Owner: bijux-dna-bench-vcf-evidence
Scope: Persisted VCF evidence records, complete binding admission, command projection and snapshots.
Allowed inputs: Typed bindings, command declarations and admitted evidence records.
Forbidden dependencies: Frontend crates bijux-dna and bijux-dna-dev; tests must not become production dependencies.
Forbidden effects: No application, planner, domain executor or filesystem orchestration dependencies.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-vcf-evidence --all-targets

[Architecture](ARCHITECTURE.md) defines the internal ownership and effect boundaries. [Public API](PUBLIC_API.md) records supported inputs and outputs. [Tests](TESTS.md) describes feature-specific qualification and evidence limits. Dependency declarations and the workspace boundary graph are executable constraints; this document does not establish new scientific or hosted performance claims.

# bijux-dna-bench-haplotypes boundary

Owner: bijux-dna-bench-haplotypes
Scope: Phasing, imputation, reference panel and imputation-metric benchmark capabilities.
Allowed inputs: Typed stage declarations, scientific inputs, admitted readiness snapshots and caller roots.
Forbidden dependencies: Frontend crates bijux-dna and bijux-dna-dev; tests must not become production dependencies.
Forbidden effects: No frontend command types or remote campaign scheduling; preserve native execution evidence.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-haplotypes --all-targets

[Architecture](ARCHITECTURE.md) defines the internal ownership and effect boundaries. [Public API](PUBLIC_API.md) records supported inputs and outputs. [Tests](TESTS.md) describes feature-specific qualification and evidence limits. Dependency declarations and the workspace boundary graph are executable constraints; this document does not establish new scientific or hosted performance claims.

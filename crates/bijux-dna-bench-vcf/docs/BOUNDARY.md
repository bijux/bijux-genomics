# bijux-dna-bench-vcf boundary

Owner: bijux-dna-bench-vcf
Scope: VCF benchmark declarations, adapter observations, evidence acquisition and readiness services.
Allowed inputs: Explicit registry/scope declarations, source providers, evidence snapshots and publication destinations.
Forbidden dependencies: Frontend crates bijux-dna and bijux-dna-dev; tests must not become production dependencies.
Forbidden effects: No frontend types or hidden global working-directory configuration; YAML is development-only.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-vcf --all-targets

[Architecture](ARCHITECTURE.md) defines the internal ownership and effect boundaries. [Public API](PUBLIC_API.md) records supported inputs and outputs. [Tests](TESTS.md) describes feature-specific qualification and evidence limits. Dependency declarations and the workspace boundary graph are executable constraints; this document does not establish new scientific or hosted performance claims.

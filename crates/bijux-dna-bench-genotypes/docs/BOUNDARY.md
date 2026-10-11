# bijux-dna-bench-genotypes boundary

Owner: bijux-dna-bench-genotypes
Scope: Admitted genotype benchmark workflows, observations, readiness and artifact publication.
Allowed inputs: Typed stage declarations, scientific settings, execution bindings and caller-owned output roots.
Forbidden dependencies: Frontend crates bijux-dna and bijux-dna-dev; tests must not become production dependencies.
Forbidden effects: No CLI parsing or implicit global environment mutation; domain execution stays behind its existing adapters.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-genotypes --all-targets

[Architecture](ARCHITECTURE.md) defines the internal ownership and effect boundaries. [Public API](PUBLIC_API.md) records supported inputs and outputs. [Tests](TESTS.md) describes feature-specific qualification and evidence limits. Dependency declarations and the workspace boundary graph are executable constraints; this document does not establish new scientific or hosted performance claims.

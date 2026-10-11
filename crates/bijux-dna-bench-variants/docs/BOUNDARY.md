# bijux-dna-bench-variants boundary

Owner: bijux-dna-bench-variants
Scope: Variant processing benchmark workflows, typed observations and readiness.
Allowed inputs: Explicit stage plans, scientific inputs and admitted retained evidence.
Forbidden dependencies: Frontend crates bijux-dna and bijux-dna-dev; tests must not become production dependencies.
Forbidden effects: No CLI parsing or duplicate domain registry; domain transformations remain in domain and stage owners.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-variants --all-targets

[Architecture](ARCHITECTURE.md) defines the internal ownership and effect boundaries. [Public API](PUBLIC_API.md) records supported inputs and outputs. [Tests](TESTS.md) describes feature-specific qualification and evidence limits. Dependency declarations and the workspace boundary graph are executable constraints; this document does not establish new scientific or hosted performance claims.

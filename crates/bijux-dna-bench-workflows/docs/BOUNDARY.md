# bijux-dna-bench-workflows boundary

Owner: bijux-dna-bench-workflows
Scope: Admitted local micro-workflow composition and report publication.
Allowed inputs: Explicit scientific workflow inputs and domain execution contracts.
Forbidden dependencies: Frontend crates bijux-dna and bijux-dna-dev; tests must not become production dependencies.
Forbidden effects: No CLI parsing or remote submission; workflow composition delegates scientific transformations to owners.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-workflows --all-targets

[Architecture](ARCHITECTURE.md) defines the internal ownership and effect boundaries. [Public API](PUBLIC_API.md) records supported inputs and outputs. [Tests](TESTS.md) describes feature-specific qualification and evidence limits. Dependency declarations and the workspace boundary graph are executable constraints; this document does not establish new scientific or hosted performance claims.

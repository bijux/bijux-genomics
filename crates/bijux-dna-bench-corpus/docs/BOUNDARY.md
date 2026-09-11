# bijux-dna-bench-corpus boundary

Owner: bijux-dna-bench-corpus
Scope: Small governed corpus fixture admission and materialization, including reference panel metadata.
Allowed inputs: Explicit source identities, fixture manifests and caller roots.
Forbidden dependencies: Frontend crates bijux-dna and bijux-dna-dev; tests must not become production dependencies.
Forbidden effects: No CLI parsing or benchmark verdict orchestration; unknown user evidence is not disposable scratch.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-corpus --all-targets

[Architecture](ARCHITECTURE.md) defines the internal ownership and effect boundaries. [Public API](PUBLIC_API.md) records supported inputs and outputs. [Tests](TESTS.md) describes feature-specific qualification and evidence limits. Dependency declarations and the workspace boundary graph are executable constraints; this document does not establish new scientific or hosted performance claims.

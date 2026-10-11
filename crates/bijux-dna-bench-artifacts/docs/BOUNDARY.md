# bijux-dna-bench-artifacts boundary

Owner: bijux-dna-bench-artifacts
Scope: Native benchmark execution evidence inventories, guarded output publication and retained physical provenance.
Allowed inputs: Typed execution paths and governed prior inventories.
Forbidden dependencies: Frontend crates bijux-dna and bijux-dna-dev; tests must not become production dependencies.
Forbidden effects: Filesystem effects must use infrastructure; no scientific verdict policy or backend scheduling.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-artifacts --all-targets

[Architecture](ARCHITECTURE.md) defines the internal ownership and effect boundaries. [Public API](PUBLIC_API.md) records supported inputs and outputs. [Tests](TESTS.md) describes feature-specific qualification and evidence limits. Dependency declarations and the workspace boundary graph are executable constraints; this document does not establish new scientific or hosted performance claims.

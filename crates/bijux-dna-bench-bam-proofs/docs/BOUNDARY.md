# BAM scientific proof boundary

Owner: bijux-dna-bench-bam-proofs
Scope: BAM benchmark scientific output admission, completion assessment and diagnostic publication.
Allowed inputs: Explicit root/output paths, canonical readiness records, typed native operation identity and domain-owned persisted summaries.
Forbidden dependencies: bijux-dna, bijux-dna-dev, bijux-dna-api, benchmark umbrella, benchmark-model, planners and runner must not enter normal or dev dependencies.
Forbidden effects: Process spawning, network access, environment mutation, global root discovery and recursive output deletion.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-bam-proofs --all-targets

Infrastructure supplies atomic per-file publication. Injected providers retain their
canonical local acquisition/publication effects; they do not certify interchangeable tools.

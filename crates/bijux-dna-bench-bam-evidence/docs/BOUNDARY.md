# BAM evidence boundary

Owner: bijux-dna-bench-bam-evidence
Scope: BAM declaration coverage, report binding admission, projections and diagnostic publication.
Allowed inputs: Explicit repository/output paths and typed canonical observations.
Forbidden dependencies: Frontend crates bijux-dna and bijux-dna-dev; native BAM, planners, API, runner, benchmark aggregation and benchmark-model are excluded from normal and dev dependencies.
Forbidden effects: No process execution, network calls, remote submission, root discovery, environment mutation or recursive output deletion. Tests must not become production dependencies.
Validation command: cargo nextest run --config-file configs/rust/nextest.toml --profile full --locked --offline -p bijux-dna-bench-bam-evidence --all-targets

Local report writes use infrastructure primitives and atomic per-file replacement.
Provider adapters may acquire and publish their canonical local evidence. Neither
provider provenance nor declaration completeness establishes fresh scientific execution.

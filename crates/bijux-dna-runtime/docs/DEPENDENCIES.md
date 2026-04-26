# Dependencies

`bijux-dna-runtime` depends only on contract types, canonical serialization, governed filesystem/runtime artifact helpers, registry/profile parsing, identity generation, hashing, timestamps, and optional telemetry integration.

## Runtime Dependencies
- `anyhow` for runtime contract and artifact I/O errors.
- `serde` and `serde_json` for stable runtime schemas, manifests, telemetry, and canonical JSON payloads.
- `sha2` for manifest, artifact, parameter, and run-layout identity hashes.
- `toml` for governed tool registry and runtime profile loading.
- `uuid` for generated run IDs.
- `chrono` for explicitly recorded runtime timestamps.
- `opentelemetry` as an optional `otel` feature dependency for telemetry spans.
- `bijux-dna-core` for shared contracts, IDs, metrics, canonical JSON, and validation errors.
- `bijux-dna-infra` for governed filesystem writes, locks, temp dirs, hash helpers, YAML parsing, and repository path helpers.

## Dev Dependencies
- `bijux-dna-policies` for guardrail checks.
- `bijux-dna-testkit` for isolated test directories.
- `walkdir` for source-tree contract tests.

## Forbidden Runtime Dependencies
- CLI/API adapter crates such as `bijux-dna` and `bijux-dna-api`.
- Analyzer and benchmark crates such as `bijux-dna-analyze` and `bijux-dna-bench`.
- Engine and planner crates such as `bijux-dna-engine`, `bijux-dna-pipelines`, `bijux-dna-planner-bam`, `bijux-dna-planner-fastq`, and `bijux-dna-planner-vcf`.
- Domain BAM/FASTQ/VCF semantics crates and stage implementation crates.
- Runner backend crates such as `bijux-dna-runner`.
- Database clients and network clients.
- Async runtimes, process execution crates, or network clients such as `tokio` and `reqwest`.

## Boundary Rule
Runtime may define handoff contracts for other layers, but it must not depend on the layers that consume those contracts.

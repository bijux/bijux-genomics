# Dependencies

`bijux-dna-policies` keeps runtime dependencies limited to pure decision models, deterministic
source scanning, manifest parsing, diagnostics, and guardrail configuration.

## Runtime Dependencies
- `anyhow` for guardrail and scanner errors.
- `chrono` for governed publication timestamps and freshness windows.
- `regex` for source-token and public-surface policy checks.
- `schemars` for policy-owned JSON Schema contracts.
- `serde` and `serde_json` for typed policy publications and scanner evidence.
- `toml` for governed config and manifest parsing support.
- `walkdir` for deterministic repository traversal.
- `sha2` for evidence and publication digests.

## Dev Dependencies
- `cargo_metadata` for dependency graph policy tests.
- `insta` for policy snapshots.
- `serde_yaml` for governed configuration checks.
- `bijux-dna-containers` for the retained registry-export snapshot contract.
- `bijux-dna-core`, `bijux-dna-pipelines`, and `bijux-dna-stage-contract` for retained repository contract checks.
- `bijux-dna-testkit` for repository discovery and fixture support.

Runtime registry parsing and stage artifact/semantic-version declarations are tested
by `bijux-dna-runtime`; policy tests do not depend on the runtime owner.

## Forbidden Runtime Dependencies
- CLI, runner, runtime, environment, API, planner, domain, stage, database, analyzer, benchmark, or pipeline crates.
- Process, network, container, database, or async runtime clients.
- `cargo_metadata` and `serde_yaml` unless production code gains an explicit runtime contract for
  them.

## Boundary Rule
If a dependency is only needed to inspect the repository in tests, it belongs in `dev-dependencies`.

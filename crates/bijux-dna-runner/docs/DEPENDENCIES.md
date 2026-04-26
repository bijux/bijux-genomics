# Dependencies

`bijux-dna-runner` depends only on execution contracts, runtime handoff types, environment image/runtime resolution, infra filesystem helpers, and deterministic identity support.

## Runtime Dependencies
- `anyhow` for execution and replay errors.
- `serde` and `serde_json` for backend specs, execution manifests, records, and runner-owned artifact payloads.
- `tracing` for backend image-resolution warnings.
- `uuid` for temporary Docker container names.
- `sha2` for invocation and input identity hashing.
- `walkdir` for deterministic directory input hashing.
- `bijux-dna-core` for execution steps, tool execution specs, IDs, hashing helpers, canonical JSON, and manifests.
- `bijux-dna-runtime` for the `Runner` trait, invocations, runner results, and artifact handoff.
- `bijux-dna-environment` for runtime kind, image catalog, image resolution, and platform contracts.
- `bijux-dna-infra` for governed filesystem writes, temp dirs, path helpers, and hashing.

## Dev Dependencies
- `bijux-dna-policies` for guardrail checks.
- `cargo_metadata` for dependency boundary tests.
- `assert_cmd` for command-runner contract tests.
- `tempfile` for isolated filesystem tests.
- `toml` for the local manifest dependency graph test.

## Forbidden Runtime Dependencies
- Engine, planner, stage, domain, API, CLI, analyzer, benchmark, and pipeline crates.
- Database clients and network clients.
- Compression, time, and async runtime crates unless a concrete runner contract requires them.

## Boundary Rule
If a dependency is only needed for tests, policy inspection, or fixture setup, it belongs in `dev-dependencies`.

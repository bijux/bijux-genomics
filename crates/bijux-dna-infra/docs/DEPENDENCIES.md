# bijux-dna-infra Dependencies

The dependency graph should stay small because this crate is consumed by many other crates.
Dependencies must support generic infrastructure only; they must not import product, domain,
planner, runner, API, database, analysis, benchmark, or environment ownership.

## Runtime Dependencies

- `fs4`: cross-platform file locking for `FileLock`.
- `serde`: shared serialization traits for config-format helpers.
- `serde_json`: JSON parsing and atomic JSON writing.
- `sha2`: SHA-256 file digest calculation.
- `tempfile`: atomic writes and managed temporary directories.
- `thiserror`: typed `IoError` implementation.
- `toml`: TOML config parsing and rendering.
- `tracing-appender`: non-blocking log writer guard returned by `init_logging`.
- `tracing-subscriber` with the `tracing` feature: subscriber installation and environment filters.
- `serde_yaml` with the `yaml` feature: YAML config compatibility.

Runtime dependencies intentionally exclude generic application error crates. Infra errors should use
the crate-local `IoError` taxonomy so callers can wrap them at their own boundary.

## Test Dependencies

- `anyhow`: shared policy test module result plumbing.
- `bijux-dna-policies`: workspace guardrails.
- `bijux-dna-testkit`: deterministic snapshot helpers.
- `insta`: public surface snapshot tests.
- `regex` and `walkdir`: local boundary scans.

## Forbidden Dependencies

Infra must not depend on workspace crates above the infrastructure layer. In particular it must not
depend on API, CLI, planner, pipeline, runner, stage, domain, database, analysis, benchmark,
environment, or science crates.

## Feature Rules

- YAML remains optional and config-only. It must not become a contract schema format.
- Tracing subscriber installation remains optional. Callers decide whether to enable tracing.
- New dependencies require a documented owner, a boundary reason, and a dependency boundary test.

## Verification

Use:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-infra --no-default-features --test boundaries
cargo tree -p bijux-dna-infra --no-default-features
```

# Commands

This file is the SSOT for callable operations managed by `bijux-dna-testkit`.
The crate owns Rust test-helper operations, not CLI commands.

## CLI Commands

None. This crate owns no binaries, subcommands, command parsing, shell
entrypoints, runner behavior, or environment provisioning commands.

## Determinism Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `create-fixed-clock` | `FixedClock::at` / `FixedClock::unix_s` | Build deterministic clock values for tests. |
| `read-fixed-clock` | `FixedClock::now` | Return the fixed time. |
| `create-fixed-rng` | `fixed_rng` | Build a seeded deterministic RNG. |
| `strip-json-timestamp-fields` | `strip_timestamp_fields` | Remove caller-selected timestamp fields from JSON. |
| `assert-json-stable` | `assert_json_stable` | Compare JSON values after stable ordering normalization. |
| `assert-stable-ordering` | `assert_stable_ordering` | Assert deterministic ordering of sorted values. |

## Fixture Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `load-fixture-text` | `load_fixture_text` | Read a UTF-8 fixture file with path-aware panic messages. |
| `load-fixture-json` | `load_fixture_json` | Read and parse a JSON fixture file with path-aware panic messages. |
| `assert-json-schema-like` | `assert_json_schema_like` | Assert that a JSON value contains schema-like top-level keys. |

## Snapshot Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `install-snapshot-env` | `install_snapshot_env` | Set deterministic timezone and locale for tests. |
| `sanitize-snapshot-text` | `sanitize_snapshot_text` | Redact host-specific text from snapshots. |
| `sanitize-snapshot-json` | `sanitize_snapshot_json` | Redact unstable JSON fields and host-specific strings. |
| `snapshot-normalize-text` | `snapshot_normalize_text` | Normalize snapshot text for assertions. |
| `snapshot-normalize-json` | `snapshot_normalize_json` | Normalize snapshot JSON for assertions. |
| `stable-json` | `stable_json` | Sort JSON object keys recursively. |
| `build-snapshot-name` | `snapshot_name` | Build stable snapshot names from bucket and test name. |

## Temporary Path Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `create-test-tempdir` | `tempdir_for` | Create an isolated temporary directory for a test. |
| `create-test-temp-path` | `temp_path_for` | Create and keep a temporary path for a test. |
| `resolve-test-path-under-root` | `resolve_under` | Resolve a contained relative path under the test temp root. |
| `create-test-paths` | `TestPaths::new` | Create a reusable test path model. |
| `read-test-path-root` | `TestPaths::root` | Return the root test path. |
| `derive-test-path-child` | `TestPaths::child` | Derive a contained child path below the test root. |
| `list-directory-sorted` | `sorted_read_dir_paths` | Read a directory and return paths in deterministic order. |

## Workspace Support Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `read-policy-text` | `read_policy_text` | Read repository policy or fixture text with path-aware panic messages. |
| `resolve-workspace-root-from-manifest` | `workspace_root_from_manifest` | Resolve the workspace root from a crate manifest directory. |

## Commands Owned Elsewhere

- User-facing CLI commands belong in command/API crates.
- Production execution belongs in runner and runtime crates.
- Domain and stage semantics belong in domain and stage crates.
- Environment and container commands belong in environment crates.

## Local Verification Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-testkit --no-default-features
```

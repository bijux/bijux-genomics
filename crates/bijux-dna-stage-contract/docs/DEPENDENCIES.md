# Dependencies

`bijux-dna-stage-contract` owns shared stage-plan and execution-step contract
types. Its dependency graph must stay pure: it may model what a stage runner
needs, but it must not execute stages, discover runtime infrastructure, or call
CLI surfaces.

## Runtime Dependencies

| Dependency | Reason |
| --- | --- |
| `anyhow` | Fallible validation and projection errors. |
| `bijux-dna-core` | Shared identifiers, artifact references, and command-contract primitives. |
| `serde` | Public contract serialization derives. |
| `serde_json` | Canonical JSON values used in params, schemas, and snapshots. |
| `sha2` | Deterministic hashes for stage plans and execution plans. |

## Dev Dependencies

| Dependency | Reason |
| --- | --- |
| `bijux-dna-policies` | Shared boundary assertions. |
| `bijux-dna-testkit` | Shared fixture and snapshot helpers. |
| `toml` | Manifest parsing in dependency-boundary tests. |
| `walkdir` | Deterministic docs, source, and test tree scans. |

## Forbidden Dependency Direction

This crate must not depend on API, engine, environment, planner, runner,
runtime, stage implementation, science, database, or CLI crates. Those layers
consume the shared contract; the contract crate must not consume their behavior.

Internal `bijux-dna-*` dependencies must be declared through the workspace catalog. The only
runtime internal edge is `bijux-dna-core`, and boundary tests require it to stay cataloged.

## Verification

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --test boundaries normal_dependency_graph_matches_stage_contract_boundary --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-stage-contract --test boundaries dev_dependency_graph_stays_policy_and_fixture_facing --no-default-features
```

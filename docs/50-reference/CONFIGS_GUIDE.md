# CONFIGS_GUIDE

## Purpose
Map `configs/` subdirectories to their contract intent and ownership boundaries.

## Scope
Covers repository config contracts under `configs/**` and their owner mapping.

## Non-goals
- Replacing per-file schema docs.
- Repeating generated registry contents.

## Contracts
- Owner mapping source of truth is [configs/OWNERS.toml](../../configs/OWNERS.toml).
- Every config path must match exactly one owner rule.
- Schema/version header checks are enforced by config schema validators.

## Directory Map
| Path Prefix | Contract Intent | Owner Source |
|---|---|---|
| `configs/ci/` | CI registries, stage/tool contracts, lock inputs | [configs/OWNERS.toml](../../configs/OWNERS.toml) |
| `configs/runtime/` | Runtime platforms, profiles, species aliases | [configs/OWNERS.toml](../../configs/OWNERS.toml) |
| `configs/schema/` | Schema policy docs and generated tree snapshot | [configs/OWNERS.toml](../../configs/OWNERS.toml) |
| `configs/docs/` | Docs toolchain pins and build config | [configs/OWNERS.toml](../../configs/OWNERS.toml) |
| `configs/hpc/` | HPC sync/profile knobs (rsync include/exclude) | [configs/OWNERS.toml](../../configs/OWNERS.toml) |
| `configs/bench/` | Benchmark execution knobs | [configs/OWNERS.toml](../../configs/OWNERS.toml) |
| `configs/coverage/` | Coverage thresholds | [configs/OWNERS.toml](../../configs/OWNERS.toml) |
| `configs/rust/` | Rust fmt, lint, audit, and nextest config | [configs/OWNERS.toml](../../configs/OWNERS.toml) |

## Validation Path
1. Run `cargo run -q -p bijux-dna-dev -- checks run check-config-owners`.
2. Run `cargo run -q -p bijux-dna-dev -- checks run check-config-schema`.
3. Run `cargo run -q -p bijux-dna-dev -- tooling run check-config-snapshot`.

# Architecture

`bijux-dna-db-ref` is a deterministic metadata resolver. Its tree is a small
root over focused namespaces:

```text
src/
├── lib.rs
├── public_api/
├── catalog/
├── model/
├── providers/
├── resolution/
└── runtime_config/
```

## Responsibilities

- `lib.rs` owns only the curated crate surface.
- `public_api/` is the explicit namespace for stable exports.
- `catalog/` owns panel and map catalog models, lock records, and compatibility policy.
- `model/` owns species authority contracts and reference asset contracts.
- `providers/` owns runtime-facing resolver traits and the default runtime implementation.
- `resolution/` owns pure lookup behavior, lock validation, and tool compatibility checks.
- `runtime_config/` owns workspace path discovery, TOML loading, and config DTOs grouped by concern.

This boundary keeps runtime loading, data contracts, and lookup behavior from collapsing back into broad root files.

## Test Tree

```text
tests/
├── boundaries.rs
├── boundaries/
│   └── architecture_tree.rs
├── contracts.rs
├── contracts/
│   └── runtime_provider.rs
└── guardrails.rs
```

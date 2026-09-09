# Architecture

`bijux-dna-db-ref` is a deterministic metadata resolver. Its tree is a small
root over focused namespaces:

```text
src/
├── lib.rs
├── public_api/
├── catalog/
├── materialization/
├── model/
├── providers/
├── resolution/
└── runtime_config/
```

## Responsibilities

- `lib.rs` owns only the curated crate surface.
- `public_api/` is the explicit namespace for stable exports.
- `catalog/` owns panel and map catalog models, lock records, and compatibility policy.
- `materialization/` owns pure source-versus-derived placement and admission planning.
- `model/` owns species authority contracts and reference asset contracts.
- `providers/` owns runtime-facing resolver traits and the default runtime implementation.
- `resolution/` owns pure lookup behavior, lock validation, and tool compatibility checks.
- `runtime_config/` owns workspace path discovery, TOML loading, and config DTOs grouped by concern.

This boundary keeps runtime loading, data contracts, and lookup behavior from collapsing back into broad root files.
It also keeps fixture generation, byte transfer, downloads, index creation, and database
materialization in their execution-owning crates rather than the metadata resolver. The resolver
owns the immutable placement decision that those executors consume.

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

`resolution/reference_catalog.rs` owns explicit-root species and bundle identity lookup through the existing complete runtime configuration types. The default species resolver delegates to the same lookup. Catalog queries reject duplicate matching identities and never fall back from a missing selected root to the compiled workspace. `tests/contracts/reference_catalog.rs` exercises isolated catalog roots and malformed declarations.

`materialization/fasta_index.rs` derives FASTA index bytes as a pure reference contract. Repository adapters own acquisition and publication. LF/CRLF offsets, final lines and per-record wrap widths are checked against pinned native index fixtures; ambiguous identities and irregular layouts are rejected before publication.

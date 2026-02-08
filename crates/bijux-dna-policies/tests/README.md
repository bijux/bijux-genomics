# Policy Tests

Directories:
- `deps/`: dependency graph and layering policies (ex: `dependency_boundaries.rs`, `effect_boundary_map.rs`).
- `surface/`: public surface and boundary guardrails (ex: `docs_spine.rs`, `no_helpers_policy.rs`, `test_grouping_policy.rs`).
- `data/`: data schema and snapshot policies (ex: `contract_handshake.rs`).
- `tooling/`: tooling and CI enforcement (ex: `docs_links.rs`, `no_appledouble.rs`).

Suite index:
- `deps/` enforces crate dependencies, effect boundaries, and infra/QA isolation.
- `surface/` enforces docs spine, README contracts, module layout, and test grouping.
- `data/` enforces contract handshake fixtures and serialized policy defaults.
- `tooling/` enforces repo hygiene and documentation link integrity.

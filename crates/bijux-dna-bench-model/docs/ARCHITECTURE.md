# bijux-dna-bench-model Architecture

`bijux-dna-bench-model` is a pure benchmark decision crate. Its ideal tree is a curated root over focused namespaces:

```text
src/
├── lib.rs
├── public_api/
├── compare/
├── contract/
├── diagnostics/
├── model/
├── policy/
└── stats/
```

Responsibilities:

- `lib.rs` owns only the curated crate surface.
- `public_api/` is the explicit namespace for stable exports.
- `compare/` owns comparison inputs, stratification, and typed comparison reports.
- `contract/` owns schema IDs and validation rules, with suite-specific validation grouped under `contract/suite/`.
- `diagnostics/` owns stable error taxonomy.
- `model/` owns benchmark decision, observation, suite, and summary contracts.
- `policy/` owns gate policy configuration, evaluation, and decision outcomes.
- `stats/` owns deterministic statistical helpers.

This boundary keeps validation, policy, diagnostics, and report contracts from collapsing back into broad root files.

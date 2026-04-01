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
- `public_api/` is the explicit namespace for stable exports through `stable_surface.rs`.
- `compare/` owns comparison inputs, stratification, and typed comparison reports, with root exports
  delegated to `stable_surface.rs`.
- `contract/` owns schema IDs and validation rules, with suite-specific validation grouped under
  `contract/suite/validation/`.
- `diagnostics/` owns stable error taxonomy.
- `model/` owns benchmark decision, observation, suite, and summary contracts, with suite support
  contracts grouped under `model/suite/support/`.
- `policy/` owns gate policy configuration, evaluation, and decision outcomes.
- `stats/` owns deterministic statistical helpers, with robust estimator contracts separated from
  estimator functions.

This boundary keeps validation, policy, diagnostics, and report contracts from collapsing back into broad root files.

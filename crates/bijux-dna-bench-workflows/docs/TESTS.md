# Workflow verification

Run default and native-feature owner tests from the workspace root:

```sh
CARGO_TARGET_DIR=artifacts/target cargo test --locked -p bijux-dna-bench-workflows --all-targets
CARGO_TARGET_DIR=artifacts/target cargo test --locked -p bijux-dna-bench-workflows --all-targets --all-features
```

Output regressions preserve unrelated files, reject path traversal before any
retirement, refuse unclaimed artifacts/repository roots and reject symbolic output
declarations. Genomics binding admission rejects missing, duplicate and malformed
observations without invoking supplied fixture operations.

Existing public CLI micro-pipeline and fixture-contract cases retain compatibility
coverage. Native stages require the repository's normal runner bindings; fixture
and semantic runtime results remain distinct from native scientific evidence.

# Adapter tests

From the workspace root, direct compiler products into the artifact directory:

```sh
CARGO_TARGET_DIR=artifacts/target cargo test --locked -p bijux-dna-bench-vcf --all-targets
CARGO_TARGET_DIR=artifacts/target cargo test --locked -p bijux-dna-bench-vcf --all-targets --all-features
```

Eleven original adapter contracts cover seven tool families and their emitted
row counts, identities and declared outputs. The contract harness obtains its
source catalog and supported pairs through the existing benchmark test provider;
that is a dev dependency only. Workspace paths resolve at runtime.

Input regressions reject absent, duplicated, foreign and malformed observations
before the injected reference writer runs. The dependency regression rejects
reverse CLI/API/benchmark edges and implicit FASTQ catalog activation. Existing
public CLI contracts separately verify stdout, persisted reports and consumers.

Native tool execution requires the usual repository runner bindings. Marker
fixtures support adapter declaration checks, not full scientific substitution.

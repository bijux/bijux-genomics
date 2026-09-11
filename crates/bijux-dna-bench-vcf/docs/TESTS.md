# Adapter tests

From the workspace root, direct compiler products into the artifact directory:

```sh
CARGO_TARGET_DIR=artifacts/target cargo test --locked -p bijux-dna-bench-vcf --all-targets
CARGO_TARGET_DIR=artifacts/target cargo test --locked -p bijux-dna-bench-vcf --all-targets --all-features
```

Eleven original adapter contracts cover seven tool families and their emitted
row counts, identities and declared outputs. The contract harness reads the existing
catalog, matrix and VCF domain declarations and supplies a fixture reference
operation using the reference owner's FASTA index builder. It does not compile
the benchmark workflow library. Production provider behavior remains covered by
public CLI tests; no static duplicate catalog or marker FASTA is introduced.
Workspace paths resolve at runtime.

Input regressions reject absent, duplicated, foreign and malformed observations
before the injected reference writer runs. The dependency regression rejects
reverse CLI/API/benchmark edges and implicit FASTQ catalog activation. Existing
public CLI contracts separately verify stdout, persisted reports and consumers.

Native tool execution requires the usual repository runner bindings. Marker
fixtures support adapter declaration checks, not full scientific substitution.

## Serving evidence contracts

`tests/contracts/serving.rs` retains the original governed matrix/report assertions.
It also proves serving projection does not invoke the reference materializer and
that an unknown parser contract fails before any output directory is created.

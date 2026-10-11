# bijux-dna-containers-qa Boundary

Owner: effectful environment QA, image readiness checks, and QA evidence persistence.

## Belongs Here

- Docker image build and smoke checks for configured tool images.
- Docker/Apptainer image QA workflows that are explicitly operator-invoked.
- Container qualification probe planning, declared-output validation, metric evaluation, and
  classified verdicts.
- Generic checksum-bound qualification specification compilation from canonical stage bindings.
- QA artifact writing under `artifacts/image-qa/<platform>/`.
- QA record persistence to JSONL, SQLite, and summary JSON.
- Offline contract tests for planning, layout, records, and fixture stability.

## Does Not Belong Here

- Process execution, scheduler lifecycle, and container image/catalog authority.
- CLI command routing outside this crate's dedicated binaries.
- Planner selection logic or domain model ownership.
- Report analysis semantics beyond writing QA evidence consumed by analyze.
- Network pulls in default tests.
- Source, config, or fixture mutation during runtime QA.

## Dependency Direction

The runner may consume this crate's stable qualification plan and verdict API while retaining all
process and scheduler effects. Other production crates must not acquire QA implementation details.
This crate may consume environment, runtime, core, FASTQ domain, infrastructure, and analyze
contracts because it is the heavy QA edge of the stack.

## Verification

Run:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers-qa --no-default-features --test boundaries
```

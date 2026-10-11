# bijux-dna-containers Tests

## Commands

Use artifact-rooted target and test temp directories:

```sh
mkdir -p artifacts/cargo-target artifacts/test-tmp
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers --no-default-features
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers --no-default-features --test boundaries
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers --no-default-features --test contracts
```

## Boundaries Suite

- `tests/boundaries/architecture.rs`: crate root, docs, source, and test layout.
- `tests/boundaries/commands.rs`: `COMMANDS.md` and process-use inventory.
- `tests/boundaries/dependencies.rs`: allowed direct dependency graph.
- `tests/boundaries/guardrails/*.rs`: workspace and runtime guardrail checks.

## Contracts Suite

- `tests/contracts/apptainer_build_and_smoke.rs`: immutable build attempts, transient retry
  classification, contract-regex smoke admission, and command failure behavior.
- `tests/contracts/tool_contract.rs`: complete catalog loading, embedded image identity,
  retained-roster closure, label binding, and runtime inspection.
- `tests/contracts/build_dockerfile.rs`: Dockerfile version ARG parsing.
- `tests/contracts/credential_reference.rs`: all tracked credential declarations, generated schema
  parity, embedded-value rejection, and requirement consistency.
- `tests/contracts/pipeline.rs`: exact v1/v2 admission across all tracked pipelines and
  cross-version field rejection.
- `tests/contracts/run_shell_capture.rs`: merged stdout/stderr behavior.
- `tests/contracts/resolve_runtime.rs`: runtime parsing, platform loading, catalog hydration, image
  resolution, cache paths, and shell capture.
- `tests/contracts/matrix/reference_matrix.rs`: image reference formatting.
- `tests/contracts/matrix/docs_reference_matrix.rs`: `ENV_REFERENCE.md` links to the matrix tests.

## Determinism Suite

- `tests/determinism/fixture_stability.rs`: stable JSON ordering for schema fixtures.

## Schemas Suite

- `tests/schemas/schema/schema_snapshots.rs`: canonical JSON snapshots for `PlatformSpec` and
  `ToolImageSpec`.

## Failure Meaning

- Boundary failures mean the docs, tree, command inventory, or dependency graph drifted.
- Contract failures mean public behavior changed.
- Schema failures mean serialized environment contracts changed.
- Determinism failures mean fixture output is no longer stable.

Qualification-registry contracts retain the complete 89-tool, 159-run namespace and its historical/tombstone cases. The existing wire fixture additionally replays legacy v2 delivery declarations, rejects unknown versions/fields and restores current bytes. The existing run-index case checks captured source size/digest and decoded runs against the canonical registry. This does not simulate concurrent file mutation or claim an atomic multi-file capture.

Shared guardrail parser regressions are owned and executed by the policy crate. This boundary suite retains its crate-specific architecture checks; it does not compile another crate's test source. Constructor-only guardrail wrappers provide no source-check coverage and are retired.

Registry projection contracts retain the complete generated container-manifest snapshot under `tests/fixtures/registry_export`. The snapshot and its case metadata move with the export owner; this small wire-format regression complements the existing synthetic projection cases and requires no container execution.

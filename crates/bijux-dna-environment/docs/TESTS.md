# bijux-dna-environment Tests

## Commands

Use artifact-rooted target and test temp directories:

```sh
mkdir -p artifacts/cargo-target artifacts/test-tmp
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment --no-default-features
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment --no-default-features --test boundaries
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment --no-default-features --test contracts
```

## Boundaries Suite

- `tests/boundaries/architecture.rs`: crate root, docs, source, and test layout.
- `tests/boundaries/commands.rs`: `COMMANDS.md` and process-use inventory.
- `tests/boundaries/dependencies.rs`: allowed direct dependency graph.
- `tests/boundaries/guardrails/*.rs`: workspace and runtime guardrail checks.

## Contracts Suite

- `tests/contracts/build_dockerfile.rs`: Dockerfile version ARG parsing.
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

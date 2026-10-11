# bijux-dna-containers-qa

## What this crate does
`bijux-dna-containers-qa` is the effectful QA edge for environment and image readiness. It builds
and probes Docker images, runs Docker/Apptainer QA workflows, writes QA evidence, and validates QA
artifacts before production layers trust image inputs. It also owns binding-derived qualification
specification compilation, pipeline-derived probe plans, and classified output/metric verdicts
consumed by the runner.

## What it must not do (boundaries)
It must not own planner selection, biological pipeline process execution, production runner
backends, scheduler state, container catalog authority, or report-analysis semantics.

## Role in the stack
This crate sits at the effectful QA boundary. It consumes environment, runtime, FASTQ-domain,
infrastructure, and analyze contracts. The runner is the sole production consumer of its stable
qualification API; API, planner, stage, and domain crates must not depend on it.

## Public API / entrypoints
See `docs/PUBLIC_API.md` and `docs/COMMANDS.md`.

Cargo binaries:

- `image_qa`
- `qa_docker_images`
- `build_docker_images`
- `runtime_verification`

## Internal module layout
The image QA surface is organized by responsibility. Start with `src/image_qa/mod.rs`, then:

- `contracts.rs`: QA stage and dataset contracts.
- `datasets/`: input discovery and subset hydration.
- `records/`: JSONL, SQLite, and summary evidence.
- `validation/`: pass requirements consumed by higher layers.
- `behavioral/`: FASTQ behavioral QA scenarios.
- `support/`: Docker runtime, output contracts, diagnostics, image resolution, and seqkit metrics.
- `qa_docker_images/`: Docker image plan/probe/report command implementation.
- `runtime_verification/`: catalog-wide Apptainer version/help probe execution.
- `qualification/`: typed probe plans, declared-output validation, metric evaluation, and failure
  classification, plus checksum-bound generic specification compilation and catalog coverage.

## Key contracts it owns/consumes
Owned contracts are documented in `docs/CONTRACTS.md`.

## Effects & determinism guarantees
Runtime QA is effectful and operator-invoked. Fast tests are offline. Effect rules live in
`docs/EFFECTS.md`; command ownership lives in `docs/COMMANDS.md`.

## How to run
Use artifact-rooted local runs:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers-qa --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -p bijux-dna-containers-qa --bin qa_docker_images -- --platform docker-amd64
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -p bijux-dna-containers-qa --bin image_qa -- --platform docker-amd64
CARGO_TARGET_DIR=artifacts/cargo-target cargo run -p bijux-dna-containers-qa --bin runtime_verification -- --help
```

## Expected runtime
Fast tests should complete quickly and stay offline. Runtime QA can take minutes to hours depending
on image count, dataset size, and local Docker/Apptainer cache state.

## Artifacts / Contracts
- QA JSONL: `artifacts/image-qa/<platform>/qa.jsonl`
- QA SQLite: `artifacts/image-qa/<platform>/qa.sqlite`
- QA summary: `artifacts/image-qa/<platform>/qa.json`
- Fixture manifest/report contracts: `tests/fixtures/qa_artifacts/default/`

## How to run its tests
See `docs/TESTS.md`.

## Where the docs live
Root docs are limited to this `README.md`. All other crate docs live in `docs/`; start at
`docs/INDEX.md`.

## Failure modes
- Boundary failures mean docs, layout, commands, or dependencies drifted.
- Contract failures mean QA artifact or runtime-helper behavior changed.
- Runtime QA failures usually indicate missing images, missing datasets, tool behavior drift, or
  Docker/Apptainer host issues.

## Stability
Contract and behavior changes follow `docs/CONTRACTS.md`, `docs/PUBLIC_API.md`, and
`docs/DEPENDENCIES.md`.

## Dependency boundary
Only `bijux-dna-runner` may consume the stable qualification facade. Keep all process execution and
scheduler effects in the runner and lock the remaining dependency direction through boundary tests.

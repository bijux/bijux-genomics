# bijux-dna-environment

## What this crate does
`bijux-dna-environment` resolves runtime platform facts, tool-image catalog entries, local cache
paths, and reference registration records for genomics execution layers.

## What it must not do (boundaries)
It must not own user-facing CLI routing, biological stage execution, planner/domain semantics,
runner backend implementation, or remote registry resolution. Bounded host command helpers are
listed in `docs/COMMANDS.md`.

## Role in the stack
Upstream consumers: API, CLI, runner, environment QA, and stage orchestration.

Downstream inputs: shared core/runtime models, infrastructure config parsing, local platform files,
local image catalogs, local registry pins, and explicit cache roots.

## Public API / entrypoints
See `docs/PUBLIC_API.md`.

Main modules:

- `build`: Docker tool defaults and Dockerfile version extraction.
- `resolve`: platform loading, catalog loading, image resolution, cache helpers, command probes,
  shell capture, smoke helpers, and reference registration.
- `runtime_spec`: compatibility checks between selected runner and platform.
- `public_api::api`: stable facade re-export.

## Resolution precedence
Inputs flow in a strict order:

1. Platform and tool catalog files are loaded from repository config paths.
2. Local registry pins hydrate missing image digests.
3. Explicit digests win over tags.
4. Cache paths are derived from runner, tool, digest or version, and architecture.

Authoritative behavior lives in `docs/ENV_REFERENCE.md`.

## Key contracts it owns/consumes
- `PlatformSpec`, `RuntimeKind`, `RuntimeSpec`
- `ToolImageSpec`, `ResolvedImage`, `ImageRef`
- `ReferenceBuildRequest`, `ReferenceRecord`, `ReferenceRegistry`
- `EnvError`

Contract and schema details live in `docs/CONTRACTS.md`.

## Effects and determinism
Declared inputs resolve deterministically. Host probes, shell capture, smoke helpers, Docker image
inspection, and requested reference-index commands are effectful and documented in
`docs/EFFECTS.md` and `docs/COMMANDS.md`.

## How to run its tests
See `docs/TESTS.md`.

High-signal commands:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment --no-default-features --test boundaries
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment --no-default-features --test contracts
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment --no-default-features
```

## Where the docs live
Root docs are limited to this `README.md`. All other crate docs live in `docs/`; start at
`docs/INDEX.md`.

## Failure modes
- Bad platform specs fail contract or schema tests.
- Missing or disabled image specs fail catalog validation.
- Command inventory drift fails boundary tests.
- Serialized model drift fails schema snapshot tests.

## Stability
Contract and behavior changes follow `docs/CONTRACTS.md`, `docs/PUBLIC_API.md`, and
`docs/DEPENDENCIES.md`.

## Where to start
- `src/runtime_spec/mod.rs`
- `src/resolve/mod.rs`
- `src/build/mod.rs`

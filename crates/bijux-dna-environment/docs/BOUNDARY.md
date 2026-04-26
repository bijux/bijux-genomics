# bijux-dna-environment Boundary

Owner: environment facts and bounded host-environment inspection.

## Belongs Here

- Runtime platform records and compatibility checks.
- Tool image catalog loading, validation, digest hydration, and deterministic image names.
- Cache root and SIF path derivation from declared environment variables.
- Dockerfile version extraction for curated tool images.
- Explicit reference registration and optional index preparation commands.
- Local runner probes and smoke-command wrappers listed in `COMMANDS.md`.

## Does Not Belong Here

- User-facing CLI parsing or command routing.
- Biological stage execution and stage output ownership.
- Planner, domain, policy-authoring, report, analysis, or benchmark semantics.
- Runner backend implementation such as Docker run argument construction.
- Network registry resolution. Registry hydration is local-file only.

## Dependency Direction

This crate may depend on shared core/runtime models and infrastructure parsing. API, CLI, runner,
environment QA, planners, stages, and domains may consume this crate, but this crate must not depend
on those higher layers.

## Effects Boundary

The crate is not purely functional: it reads local files, reads environment variables, probes host
commands, checks Docker image presence, and can run requested reference-index commands. Those effects
are bounded, documented in `EFFECTS.md`, and covered by `COMMANDS.md`. It must not execute pipeline
steps or mutate repository state.

## Verification

Run:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment --no-default-features --test boundaries
```

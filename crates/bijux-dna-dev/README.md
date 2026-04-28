# bijux-dna-dev

## What this crate does
Owns the workspace development control plane for repository checks, generated metadata, container governance, and maintenance automation.

## What it must not do (boundaries)
It must not own production FASTQ, BAM, or VCF planning, runtime stage execution, or domain semantics that belong in domain, planner, runtime, or engine crates.

## Effects & determinism guarantees
This crate is allowed to perform explicit repository filesystem writes and process execution for developer automation. Commands must keep outputs deterministic, route writes through governed locations, and document any external side effects in crate docs.

## Public API / entrypoints
Start with [docs/INDEX.md](docs/INDEX.md), [docs/TESTS.md](docs/TESTS.md), [docs/BOUNDARY.md](docs/BOUNDARY.md), and [docs/PUBLIC_API.md](docs/PUBLIC_API.md). The binary entrypoint is `src/main.rs`, the stable crate-local launcher lives in `src/dev_entrypoint.rs`, CLI routing lives under `src/cli/`, and command implementations live under `src/commands/`.

## Key contracts it owns/consumes
It owns the development automation surface, generated-document workflows, container-control commands, domain-governance automation, and repository checks that are intentionally outside production pipeline execution. It consumes workspace policy, domain registry, and generated-config contracts.

## Artifacts / Contracts
Owned outputs include governed automation reports, generated docs, config snapshots, lock metadata, and container-control summaries written under repository artifact roots. Contract details live in [docs/CONTRACTS.md](docs/CONTRACTS.md), [docs/DEPENDENCIES.md](docs/DEPENDENCIES.md), and [docs/TESTS.md](docs/TESTS.md).

## Failure modes
Failures surface as explicit command errors, repository drift reports, contract-check failures, and guardrail violations when automation would mutate or read outside approved surfaces.

## How to run its tests
See [docs/TESTS.md](docs/TESTS.md). Key coverage starts in `tests/boundaries.rs`, `tests/boundaries/architecture.rs`, `tests/boundaries/guardrails.rs`, `src/commands/repo_checks.rs`, and `src/commands/containers/runtime/frontend_proofs.rs`.

## Where the docs live
Start at [docs/INDEX.md](docs/INDEX.md), then read [docs/SCOPE.md](docs/SCOPE.md), [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md), [docs/COMMANDS.md](docs/COMMANDS.md), [docs/CONTRACTS.md](docs/CONTRACTS.md), and [docs/TESTS.md](docs/TESTS.md).

## Workspace Policy
Workspace work on this crate is governed by `/Users/bijan/bijux/bijux-genomics/README.md`,
`/Users/bijan/bijux/README.md`, and `/Users/bijan/bijux/CODEX.md`; re-read
those files before editing this child repository or making commits.

# bijux-dna-runner Docs Index

This directory is the single documentation home for `bijux-dna-runner`. The crate root keeps only `README.md`; every other crate doc belongs here.

## Core Contracts
- `ARCHITECTURE.md` maps source layout and ownership.
- `BOUNDARY.md` defines what the runner may and may not own.
- `PUBLIC_API.md` lists stable modules, root exports, and facade exports.
- `EXECUTION_SPEC.md` documents backend execution semantics, failure handling, and backend invariants.

## Operational Boundaries
- `COMMANDS.md` is the single source of truth for commands this crate can manage.
- `DEPENDENCIES.md` documents allowed runtime and dev dependencies.
- `EFFECTS.md` documents process, filesystem, environment, and network effects.
- `DETERMINISM.md` documents replay, invocation identity, and stable-output guarantees.

## Maintenance
- `TESTS.md` maps test suites to the contracts they enforce.

## Change Rules
- Keep docs and tests together when changing a runner contract.
- Keep runtime responsibilities limited to resolved execution, artifact capture, and replay verification.
- Add dependencies only when they fit `DEPENDENCIES.md` and update the dependency boundary test.
- Add commands only when they fit `COMMANDS.md` and update the command inventory test.

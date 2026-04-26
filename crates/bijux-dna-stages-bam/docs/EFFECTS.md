# Effects

This crate is a pure stage-contract and observer library. Production code may
read existing files named by a stage plan or output artifact list; it must not
create run products or execute tools.

## Allowed Production Effects

- Read existing BAM observer output files.
- Read existing stage input files for stable input fingerprints.
- Build deterministic invocation and metrics envelope values in memory.
- Parse supported tool outputs into BAM domain metric structures.

## Forbidden Production Effects

- Process spawning, shell execution, container execution, or tool installation.
- Network access.
- Runtime scheduling, retries, or cancellation.
- Environment setup or container image resolution.
- Filesystem writes from production code.

## Test Effects

Tests may write temporary fixture copies and update snapshots only when the
explicit `UPDATE_CONTRACTS=1` workflow is used. Temporary outputs must be
created through repository-approved helpers and local command output must stay
under `artifacts/` when cargo is invoked.

## Enforcement

- `tests/boundaries/purity.rs` rejects command construction and tool selection.
- `tests/boundaries/pipeline_guardrails.rs` rejects pipeline composition.
- `tests/contracts/observer/observer_determinism.rs` and snapshot tests enforce stable output.

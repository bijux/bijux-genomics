# Effects

`bijux-dna-planner-vcf` is a planning crate. Its output may contain command specs, but this crate must not execute them.

## Allowed
- Pure plan construction.
- Reading repository-owned registry files under `configs/ci/`.
- Reading reference catalog views through DB-ref APIs.
- Deterministic serialization and snapshot generation during tests.

## Forbidden
- Process spawning.
- Runtime tool discovery.
- Network access.
- Product execution.
- CLI parsing or command routing.
- Tool-output parsing.
- Generated configuration mutation.

## Enforcement
- Shared policy guardrails run through `tests/guardrails.rs`.
- Planned command ownership is documented in `docs/COMMANDS.md`.
- Production source effect checks live in `tests/boundaries/source_effects.rs`.

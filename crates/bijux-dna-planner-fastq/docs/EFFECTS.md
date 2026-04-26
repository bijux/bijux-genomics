# Effects

`bijux-dna-planner-fastq` is a planning crate. Its output may contain command specs, but this crate must not execute them.

## Allowed
- Pure plan construction.
- Reading repository-owned tool registry and fixture configuration.
- Deterministic serialization and snapshot generation during tests.
- Tracing plan graph metadata.

## Forbidden
- Process spawning.
- Runtime tool discovery.
- Network access.
- Product execution.
- CLI parsing or command routing.
- Tool-output parsing.
- Generated configuration mutation.

## Enforcement
- Shared policy guardrails run through `tests/boundaries.rs`.
- Planner-specific purity checks live under `tests/contracts/plan/no_parsing.rs`.
- `docs/COMMANDS.md` records that this crate owns planned command specs, not runtime commands.

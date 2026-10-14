# Effects

`bijux-dna-planner-bam` is a planning crate. Its output may contain command specs, but this crate must not execute them.

## Allowed
- Pure plan construction.
- Reading repository-owned tool registry configuration.
- Reading an explicit `BIJUX_REPO_ROOT` binding to select validated Cargo workspace
  and `bijux-dna` package authority. An invalid binding or missing or malformed
  explicitly bound registry admits no candidates. Default selection refuses
  those failures rather than consulting the build checkout.
- Deterministic serialization and snapshot generation during tests.
- Tracing plan graph metadata.

Without a binding, a valid same-repository working location selects that
repository; an external working location uses the validated repository default
associated with the build. Each default selection uses one owned registry
observation for both candidate admission and primary choice. Registry state is
not cached across roots or operations. Without an explicit binding, an absent,
unreadable, or unparsable optional registry retains canonical domain fallback.
Explicit bindings always require readable, parseable registry authority.

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
- Source effect boundaries are locked by `tests/boundaries/source_effects.rs`.
- Planner-specific purity checks live under `tests/contracts/plan/no_parsing_execution.rs`.
- `docs/COMMANDS.md` records that this crate owns planned command specs, not runtime commands.

# Effects

`bijux-dna-runtime` is a governed filesystem and contract-writing boundary. It may create run-layout directories, write runtime-owned artifacts, append JSONL events, read governed registry/profile files, and record timestamps. It must not execute tools or perform hidden network work.

## Allowed Effects
- Create directories under declared run/layout roots.
- Write canonical JSON runtime artifacts.
- Append runtime-owned JSONL event files.
- Write bounded execution log captures provided by runner callers.
- Acquire file locks for runtime journal and telemetry writes.
- Read governed runtime profiles, registry files, fixtures, and declared artifact inputs.
- Hash declared artifacts.
- Record explicit timestamps through runtime contracts.
- Build a telemetry adapter; optional OpenTelemetry spans require the `otel` feature and `BIJUX_OTEL=1`.

## Forbidden Effects
- No process spawning.
- No Docker or Apptainer invocation.
- No network access.
- No CLI parsing.
- No planner, stage implementation, analyzer, or benchmark effects.
- No writes outside declared run-layout or tool-run roots.

## Determinism Rules
- Canonical JSON writers are required for stable runtime contracts.
- Runtime timestamps are explicit contract fields and are unstable across real runs.
- Fixture snapshots pin schema shape, not wall-clock values.

## Validation
- `tests/boundaries/guardrails.rs` runs policy guardrails.
- `tests/contracts/canonical_writer.rs` checks runtime-owned JSON emitters.
- `tests/contracts/manifest_integrity.rs` checks manifest and artifact writer integrity.
- `tests/determinism/fixture_stability.rs` checks fixture stability.

## Failure modes
- Forbidden effects fail guardrail or boundary tests.
- Writer drift fails contract, snapshot, or canonical writer tests.

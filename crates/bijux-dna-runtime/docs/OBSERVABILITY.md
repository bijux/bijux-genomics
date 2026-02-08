# OBSERVABILITY

## Event model
Events are schema-stable, append-only records describing execution progress.
They are written to `events.jsonl` under the run layout and validated against
the runtime event schema.

## Ownership and emission
- **Runtime**: owns event schema and canonical serialization.
- **Runner**: emits concrete events during tool execution.
- **Engine**: orchestrates and wires event emission, but does not author schemas.

## Enforced by
- `tests/schema/runtime_schema_snapshots.rs`
- `tests/contracts/manifest_integrity.rs`

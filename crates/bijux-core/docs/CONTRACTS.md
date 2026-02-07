# CONTRACTS

## Contract Atlas
Each contract type includes purpose, stability tier, versioning rules, canonical JSON example, and compatibility breaks.

### ExecutionGraph
- Purpose: declarative execution plan for the engine.
- Stability tier: **A** (strict stability).
- Versioning: additive fields => minor; breaking changes => major.
- Canonical JSON example:
```json
{
  "schema_version": "bijux.execution_graph.v1",
  "contract_version": {"major": 1, "minor": 0},
  "pipeline_id": "fastq.default.v1",
  "planner_version": "planner.v1",
  "policy": {"mode": "strict"},
  "steps": [],
  "edges": []
}
```
- Breaks compatibility: renaming fields, changing step/edge semantics, removing required fields.

### RunManifest
- Purpose: canonical record of what ran and what artifacts exist.
- Stability tier: **A**.
- Versioning: additive optional fields => minor; breaking fields => major.
- Canonical JSON example:
```json
{
  "schema_version": "bijux.run_manifest.v1",
  "contract_version": {"major": 1, "minor": 0},
  "graph_hash": "sha256:...",
  "artifacts": []
}
```
- Breaks compatibility: changes to artifact list semantics or tool identity fields.

### ToolInvocation
- Purpose: exact tool identity + params + input hashes.
- Stability tier: **A**.
- Versioning: additive fields => minor.
- Canonical JSON example:
```json
{
  "tool_id": "fastp",
  "tool_version": "0.23.2",
  "image_digest": "sha256:...",
  "params_hash": "sha256:...",
  "input_hash": "sha256:..."
}
```
- Breaks compatibility: changing hashing inputs or identity fields.

### MetricsEnvelope
- Purpose: strongly typed metrics container.
- Stability tier: **B** (semantics stable, fields additive).
- Versioning: new metrics => minor.
- Canonical JSON example:
```json
{
  "tool_id": "fastp",
  "metrics": {}
}
```
- Breaks compatibility: changing required metric meanings or units.

## Non-goals
- Tool execution, process spawning, or filesystem effects.
- Domain-specific selection logic.

## See also
- `docs/SERIALIZATION.md`
- `docs/INVARIANTS.md`
- `docs/PUBLIC_API.md`

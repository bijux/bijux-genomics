# SCHEMAS

Each public type has a stable JSON shape. Examples below are canonical.

## StagePlanV1
```json
{
  "schema_version": "bijux.stage_plan.v1",
  "stage_id": "fastq.trim",
  "inputs": [],
  "outputs": []
}
```

## ExecutionPlanV1
```json
{
  "schema_version": "bijux.execution_plan.v1",
  "steps": [],
  "edges": []
}
```

## StagePluginOutputV1
```json
{
  "schema_version": "bijux.stage_plugin_output.v1",
  "metrics": {}
}
```

Stability: additive fields are backward-compatible; breaking changes require a major bump.

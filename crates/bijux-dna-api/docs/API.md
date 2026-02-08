# API

## Authoritative Surface
The v1 API is the single public surface. All schemas are stable and versioned.

## Endpoints
- `plan` → `PlanResponse`
- `execute` → `ExecuteResponse`
- `dry-run` → `DryRunResponse`
- `status` → `RunStatus`
- `explain` → `ExplainResponse`
- `policy-audit` → policy audit JSON

## Schemas
- PlanResponse (graph + graph hash)
- ExecuteResponse (run id + manifest + report pointer)
- DryRunResponse (manifest + graph)
- RunStatus (status + metadata)
- ExplainResponse (tool selection + defaults diff)

## Example
```
POST /v1/plan
{ "pipeline": "fastq.default.v1" }
```

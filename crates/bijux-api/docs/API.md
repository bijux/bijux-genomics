# API

## Authoritative Surface
The v1 API is the single public surface. All schemas are stable and versioned.

## Schemas
- PlanResponse (graph + graph hash)
- ExecuteResponse (run id + manifest + report pointer)
- ExplainResponse (tool selection + defaults diff)

## Example
```
POST /v1/plan
{ "pipeline": "fastq.default.v1" }
```

# ENDPOINT_GUIDES

## plan
- Request schema: `PlanRequest`
- Response schema: `PlanResponse`
- Stability: additive fields only; breaking changes bump major.
- Determinism: same inputs -> same graph hash.
- Failure codes: `PlanError`, `ValidationError`.

## execute
- Request schema: `ExecuteRequest`
- Response schema: `ExecuteResponse`
- Stability: additive fields only; breaking changes bump major.
- Determinism: same inputs -> same run manifest hash (timestamps excluded).
- Failure codes: `ToolError`, `ContractError`, `InfraError`.

## report
- Request schema: `ReportRequest`
- Response schema: `ReportResponse`
- Stability: additive fields only; breaking changes bump major.
- Determinism: same inputs -> same report JSON (timestamps excluded).
- Failure codes: `ParseError`, `ContractError`.

## run-index
- Request schema: `RunIndexRequest`
- Response schema: `RunIndexResponse`
- Stability: additive fields only; breaking changes bump major.
- Determinism: stable ordering.
- Failure codes: `InfraError`.

## explain
- Request schema: `ExplainRequest`
- Response schema: `ExplainResponse`
- Stability: additive fields only; breaking changes bump major.
- Determinism: same inputs -> same explain payload.
- Failure codes: `PlanError`.

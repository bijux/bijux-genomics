# Benchmark Contract

## Inputs
- Runtime run records (`bijux.run_record.v1`)
- Analyze report outputs (`bijux.report.v1`)

## Outputs
- Benchmark suite summaries (`bijux.bench.summary.v1`)
- Gate decisions (`bijux.bench.gate_decision.v1`)

Benchmark inputs are validated against their schemas and must remain compatible with the runtime
and analyze contracts. All outputs are deterministic and canonicalized.

# QUICKSTART

## What
A copy/paste walkthrough that produces a first successful run and artifacts.

## Why
Provides a minimal end-to-end proof that Bijux is installed and functioning.

## Non-goals
- Benchmark-quality runs.
- Performance tuning.

## Contracts
All outputs are contract artifacts: manifest, report, and step records.
If a command touches Cargo, run it under the shared `artifacts/` environment.
Canonical pipeline presets live in [../50-reference/PIPELINES.md](../50-reference/PIPELINES.md).

## Examples
```bash
# Plan + execute a minimal FASTQ pipeline
mkdir -p artifacts/quickstart
cargo run --bin bijux-dna -- plan --pipeline fastq.default.v1 > artifacts/quickstart/graph.json
cargo run --bin bijux-dna -- execute --pipeline fastq.default.v1 --out artifacts/quickstart/run
```

Artifacts created:
- `artifacts/quickstart/run/run_manifest.json`
- `artifacts/quickstart/run/report.json`
- `artifacts/quickstart/run/report.html`
- `artifacts/quickstart/run/summary.tsv`
- `artifacts/quickstart/run/stage_0/*`

See [../30-operations/RUN_ARTIFACTS.md](../30-operations/RUN_ARTIFACTS.md) for artifact meanings.

## Failure modes
- Missing tools => `ToolError` in execution.
- Missing artifacts => `ContractError` after step execution.

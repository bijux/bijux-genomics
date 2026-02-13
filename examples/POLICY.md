# Examples Policy

Purpose:
- Define enforceable rules for example structure, execution, and optional notebook usage.

Scope:
- Applies to all runnable example directories under `examples/` except `examples/_template` and `examples/data`.

Contracts:
- Each example is listed in `examples/index.yaml`.
- Each example is runnable via `./scripts/examples/run.sh <example-id>`.
- Golden outputs (`plan.json`, `explain.json`, `report.json`) are required and validated.

## Notebook Optional Path Rule
- Notebooks are optional convenience artifacts only.
- No example correctness contract may require `notebook.ipynb`.
- Every notebook-bearing example must document that outputs are reproducible from CLI commands.
- Notebook files must be explicitly listed in `examples/notebooks_allowlist.txt`.
- Policy: no notebooks unless allowlisted in `examples/notebooks_allowlist.txt`.

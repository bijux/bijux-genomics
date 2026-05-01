# Examples Policy

Purpose:
- Define enforceable rules for example structure, execution, and optional notebook usage.

Scope:
- Applies to runnable example directories that ship an `example.toml`.
- Applies to refusal bundles under `examples/failures/`.
- Applies to recipe-only benchmark documentation directories listed in `examples/RECIPE_ONLY.txt`.
- Applies to corpora directories under `examples/data/` only where they are referenced by runnable examples.

Contracts:
- Runnable examples:
  - Are listed in `examples/index.yaml`.
  - Are runnable via `cargo run -q -p bijux-dna-dev -- examples run run <example-id>`.
  - Carry `README.md`, `example.toml`, and golden outputs (`plan.json`, `explain.json`, `report.json`).
  - When `canonical_example: true`, also carry `tiny-inputs.json`, `workflow-manifest.json`, and `expected-evidence.json`.
- Refusal bundles:
  - Live under `examples/failures/`.
  - Must carry `README.md` and `refusal-bundle.json`.
  - Must not be listed in `examples/index.yaml`; refusal bundles must not be listed in `examples/index.yaml`.
- Recipe-only benchmark documentation:
  - Must be listed in `examples/RECIPE_ONLY.txt`.
  - Must stay `README.md`-only until promoted into a runnable example.
  - Must document commands that already exist in the shipped CLI surface.
- Corpora directories are input assets, not runnable examples, and must not be added to `examples/index.yaml`.

## Notebook Optional Path Rule
- Notebooks are optional convenience artifacts only.
- No example correctness contract may require `notebook.ipynb`.
- Every notebook-bearing example must document that outputs are reproducible from CLI commands.
- Notebook files must be explicitly listed in `examples/notebooks_allowlist.txt`.
- Policy: no notebooks unless allowlisted in `examples/notebooks_allowlist.txt`.

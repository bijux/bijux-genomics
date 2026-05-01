# Examples Index

Canonical example index: `examples/index.yaml` (generated).

## Example Classes

- Runnable examples: carry `example.toml`, golden outputs, and a generated entry in `examples/index.yaml`.
- Canonical essential examples: are the runnable entries where `canonical_example: true` in `examples/index.yaml`; inspect their contract files in-place instead of re-listing them here.
- Recipe-only benchmark docs: live under domain folders, are listed in `examples/RECIPE_ONLY.txt`, and remain `README.md`-only until the CLI contract is ready.
- Data corpora: live under `examples/data/` and provide reproducible inputs for runnable examples.
- `_template`: lives under `examples/_template/` and is an authoring scaffold, not a runnable example ID.

## Runnable Inventory

- Use `examples/index.yaml` as the only runnable example inventory.
- Do not duplicate runnable example IDs manually in this file; they drifted before.

## Corpora Inputs

- `examples/data/corpus-01/`
- `examples/data/corpus-01-mini/`

## Recipe-Only Benchmark Docs

- `examples/fastq/index-reference-bench/`
- `examples/fastq/normalize-abundance-bench/`

## Contracts

- Treat `examples/index.yaml` as the SSOT for runnable example IDs.
- Treat per-example `tiny-inputs.json`, `workflow-manifest.json`, and `expected-evidence.json` as the SSOT for canonical example inputs, stage order, and evidence expectations.
- Treat `examples/POLICY.md` as the boundary contract for runnable examples, recipe-only benchmark docs, and notebook usage.

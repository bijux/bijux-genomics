# Assets Taxonomy

## What
`assets/` stores deterministic scientific data artifacts consumed by the workspace.

## Why
A single taxonomy keeps ownership clear, prevents root-level sprawl, and separates production reference data from testing fixtures.

## Structure
- `assets/publications/`: publication-scoped artifacts and metadata.
- `assets/reference/`: reusable banks/references used by production domains.
- `assets/toy/`: minimal deterministic toy datasets for smoke/tests.
- `assets/golden/`: expected outputs and golden fixtures used by contracts.

## Rules
- Do not store executable code in `assets/`.
- Toy data must live under `assets/toy/<dataset-id>/`.
- Golden artifacts must include deterministic regeneration guidance.
- Publication folders must include `MANIFEST.toml`.
- Publication metadata is authored and reviewed manually until a dedicated publication refresh flow exists.
- Global rules are enforced by `assets/CONTRACT.md`.

---
Asset Provenance Footer
Last regenerated: 2026-02-13
Managed commands: `cargo run -p bijux-dna-dev -- assets run refresh-reference`, `cargo run -p bijux-dna-dev -- assets run refresh-toy`, `cargo run -p bijux-dna-dev -- assets run refresh-golden`

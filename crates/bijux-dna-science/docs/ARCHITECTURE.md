# bijux-dna-science Architecture

## Overview
The crate is a pure science-control surface with a thin CLI shell around deterministic loaders,
compilers, renderers, and release writers.

## Module Layout
- `app` coordinates CLI commands and write destinations.
- `cli` defines command-line arguments.
- `compile` loads authored YAML specs and derives evidence tables.
- `domain` owns science data structures and typed science identifiers.
- `errors` formats validation failures.
- `io` provides deterministic UTF-8 file IO helpers.
- `release` writes immutable release bundles under `artifacts/`.
- `render` converts compiled science rows to stable TSV and JSON.
- `schema` declares accepted authored spec versions.

## Boundaries
This crate must not execute tools, choose pipeline routes, or own runtime policy. Those concerns
remain in planner, engine, runner, and environment crates.

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

## Data Flow

1. `cli` parses the command and workspace root.
2. `app` dispatches the command and chooses whether files may be written.
3. `compile` loads authored specs and governed upstream evidence, validates cross
   references, and produces `CompiledScience`.
4. `render` converts compiled rows into stable TSV and JSON text.
5. `app` writes governed generated outputs for `build`; `release` writes immutable
   release bundles for `release`.

## Source Layout Contract

The crate root should stay small: `Cargo.toml`, `README.md`, `docs/`, `src/`, and
`tests/`. Source modules are grouped by responsibility, not by command name. Pure
domain structs remain in `domain`; filesystem writes stay in `app`, `io`, and
`release`; rendering stays in `render`.

## Boundaries

This crate must not execute tools, choose pipeline routes, or own runtime policy. Those concerns
remain in planner, engine, runner, and environment crates.

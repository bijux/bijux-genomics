# bijux-dna-science Architecture

## Overview

The crate is a pure science-control surface with a thin CLI shell around deterministic loaders,
compilers, renderers, and release writers.

## Module Layout

```text
src/
├── app/
│   └── mod.rs       # CLI command coordination and governed write destinations
├── cli.rs           # command-line arguments and subcommands
├── compile.rs       # authored YAML loading, cross-reference checks, and evidence derivation
├── decision/
│   ├── benchmark_gate.rs # fail-closed benchmark science gate evaluation
│   └── mod.rs             # scientific decision facade
├── domain/
│   └── mod.rs       # science data structures and typed science identifiers
├── evidence/
│   ├── fastq_read_lengths.rs # bounded independent FASTQ truth derivation
│   └── mod.rs               # scientific evidence derivation facade
├── errors.rs        # validation error formatting
├── io.rs            # deterministic UTF-8 file IO helpers
├── lib.rs           # library exports
├── main.rs          # binary shell
├── release.rs       # immutable release bundle writer under artifacts/
├── render/
│   └── mod.rs       # stable TSV and JSON rendering
└── schema/
    └── mod.rs       # accepted authored spec versions
```

## Data Flow

1. `cli` parses the command and workspace root.
2. `app` dispatches the command and chooses whether files may be written.
3. `compile` loads authored specs and governed upstream evidence, validates cross
   references, and produces `CompiledScience`.
4. `decision` evaluates measured benchmark evidence without reading or writing runtime state.
5. `render` converts compiled rows into stable TSV and JSON text.
6. `app` writes governed generated outputs for `build`; `release` writes immutable
   release bundles for `release`.

## Source Layout Contract

The crate root should stay small: `Cargo.toml`, `README.md`, `docs/`, `src/`, and
`tests/`. Source modules are grouped by responsibility, not by command name. Pure
domain structs remain in `domain`; filesystem writes stay in `app`, `io`, and
`release`; rendering stays in `render`.

## Boundaries

This crate must not execute tools, choose pipeline routes, or own runtime policy. Those concerns
remain in planner, engine, runner, and environment crates.

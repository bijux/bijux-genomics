# Architecture

## Goals
- keep the library root thin and explicit
- separate binary launch, CLI assembly, client protocol handling, download execution, and model
  contracts by enduring concern
- preserve stable `client`, `download`, and `model` module paths while curating a higher-level
  public API facade

## Source tree

```text
src/
├── cli/
│   ├── args.rs
│   ├── commands/
│   │   ├── download.rs
│   │   ├── mod.rs
│   │   └── query.rs
│   └── mod.rs
├── cli_entrypoint.rs
├── client/
│   ├── error.rs
│   ├── filereport/
│   │   ├── headers.rs
│   │   ├── mod.rs
│   │   ├── request.rs
│   │   └── rows.rs
│   └── mod.rs
├── download/
│   ├── config.rs
│   ├── mod.rs
│   ├── output_layout.rs
│   ├── plan.rs
│   ├── report.rs
│   ├── runtime.rs
│   ├── task.rs
│   └── transfer.rs
├── lib.rs
├── main.rs
├── manifest_store.rs
├── model/
│   ├── manifest.rs
│   ├── mod.rs
│   ├── query.rs
│   ├── record.rs
│   └── source_selection.rs
└── public_api/
    └── mod.rs
```

## Responsibilities
- `lib.rs`: stable module exports plus the curated API facade
- `public_api/`: durable high-level re-exports
- `cli_entrypoint.rs`: binary handoff only
- `cli/commands/`: command-specific manifest and download assembly
- `client/filereport/`: request-field contracts, header validation, and row decoding
- `download/runtime.rs`: pool and HTTP runtime setup
- `download/transfer.rs`: retried file transfer loop
- `download/output_layout.rs`: output path naming policy
- `model/source_selection.rs`: ENA result/source selection contracts

## Change rules
- add new root files only for enduring crate-level concerns
- prefer focused submodules over growing `client/mod.rs`, `download/mod.rs`, or `cli/mod.rs`
- update this map and the architecture boundary test together when the layout changes intentionally

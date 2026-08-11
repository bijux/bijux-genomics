# Architecture

## Goals

- keep the library root thin and explicit
- separate binary launch, CLI assembly, client protocol handling, download
  execution, and model contracts by enduring concern
- preserve stable `client`, `download`, and `model` module paths while curating
  a higher-level public API facade

## Source layout

- `lib.rs` and `public_api/` expose the curated library surface.
- `cli_entrypoint.rs` and `cli/` own the binary handoff and command assembly.
- `client/filereport/` owns request fields, response headers, and row decoding.
- `download/` owns planning, bounded transfer, checksums, and output layout.
- `model/` owns manifests, queries, records, and source-selection contracts.
- `manifest_store.rs` owns persisted manifest admission.

## Responsibilities

- `lib.rs`: stable module exports plus the curated API facade
- `public_api/`: durable high-level re-exports
- `cli_entrypoint.rs`: binary handoff only
- `cli/commands/`: command-specific manifest and download assembly
- `client/filereport/`: request-field contracts, header validation, and row decoding
- `download/runtime.rs`: pool and HTTP runtime setup
- `download/transfer.rs`: retried file transfer loop
- `download/verified_transfer.rs`: checksum-pinned, no-clobber governed transfer
- `download/output_layout.rs`: output path naming policy
- `model/source_selection.rs`: ENA result/source selection contracts

## Change rules

- add new root files only for enduring crate-level concerns
- prefer focused submodules over growing `client/mod.rs`, `download/mod.rs`, or `cli/mod.rs`
- update this map and the architecture boundary test together when the layout changes intentionally

## Verification

`tests/boundaries/architecture.rs` protects the durable source layout; the
crate-level boundary and guardrail suites protect public ownership and effects.

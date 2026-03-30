# ARCHITECTURE

## Goals
- Keep the library root thin and stable.
- Separate ENA domain models, request parsing, and download execution by concern.
- Keep the binary entrypoint thin by delegating CLI workflow into dedicated modules.

## Source tree

```text
src/
├── cli/
│   ├── args.rs
│   ├── manifest.rs
│   ├── mod.rs
│   └── workflow.rs
├── client/
│   ├── mod.rs
│   ├── parse.rs
│   └── request.rs
├── download/
│   ├── mod.rs
│   ├── planning.rs
│   └── transfer.rs
├── lib.rs
├── main.rs
├── model/
│   ├── mod.rs
│   ├── query.rs
│   └── record.rs
└── surface.rs
```

## Responsibilities
- `surface.rs`: public library exports.
- `model/`: ENA query, source, record, and manifest domain types.
- `client/request.rs`: filereport URL and field selection.
- `client/parse.rs`: TSV decoding into normalized records.
- `download/planning.rs`: deterministic download task planning.
- `download/transfer.rs`: parallel transfer execution and reporting.
- `cli/`: binary-only argument parsing, workflow orchestration, and manifest writing.

## Change rules
- Add new root files only for enduring top-level concerns.
- Prefer explicit submodules over growing `client/mod.rs`, `download/mod.rs`, or `main.rs`.
- Update this document and the tree contract together when the layout changes intentionally.

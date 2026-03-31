# Architecture

## Goals
- Keep the library root thin and stable.
- Separate ENA domain models, request parsing, and download execution by concern.
- Keep the binary entrypoint thin by delegating CLI command dispatch into dedicated modules.

## Source tree

```text
src/
├── cli/
│   ├── args.rs
│   ├── dispatch.rs
│   ├── manifest.rs
│   └── mod.rs
├── client/
│   ├── error.rs
│   ├── mod.rs
│   ├── parse.rs
│   └── request.rs
├── download/
│   ├── config.rs
│   ├── execute.rs
│   ├── item.rs
│   ├── mod.rs
│   ├── plan.rs
│   └── report.rs
├── lib.rs
├── main.rs
├── model/
│   ├── manifest.rs
│   ├── mod.rs
│   ├── query.rs
│   └── record.rs
```

## Responsibilities
- `lib.rs`: public library exports and enduring library entrypoint.
- `model/`: ENA query, manifest, source, and record domain types.
- `client/error.rs`: ENA client failure contract.
- `client/request.rs`: filereport URL and field selection.
- `client/parse.rs`: TSV decoding into normalized records.
- `download/config.rs`: transfer configuration defaults and serialization.
- `download/item.rs`: one planned file download.
- `download/plan.rs`: deterministic download planning.
- `download/execute.rs`: parallel download execution.
- `download/report.rs`: transfer outcome summary.
- `cli/`: binary-only argument parsing, command dispatch, and manifest writing.

## Change rules
- Add new root files only for enduring crate-level concerns.
- Prefer focused submodules over growing `main.rs`, `client/mod.rs`, or `download/mod.rs`.
- Update this map and the boundary tree contract together when the layout changes intentionally.

## Pointers
- `INDEX.md` for the documentation map.
- `SCOPE.md` for crate boundaries.
- `TESTS.md` for verification coverage.

# Commands

This crate owns no runtime CLI commands.

## Managed Commands

None. `bijux-dna-pipelines` is a library crate that declares pipeline profiles, defaults ledgers, and registry lookup contracts for downstream planners and applications.

## Command Boundaries
- Do not add `src/bin/` entrypoints to this crate.
- Do not parse CLI arguments here.
- Do not spawn tools, shells, or subprocesses here.
- Command naming, routing, and user-facing invocation belong to downstream command-surface crates.

## Validation
Run:

```bash
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-pipelines --no-default-features --test boundaries command_inventory_documents_no_runtime_commands
```

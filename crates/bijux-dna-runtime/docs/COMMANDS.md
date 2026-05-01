# Commands

`bijux-dna-runtime` is a library crate. It does not expose Cargo binary targets or a CLI surface, and it must not spawn backend commands.

## Runtime Commands
None.

## Managed Command Inventory

### Command Families
None.

### Runtime Entry Points
These are library functions, not shell commands:

- `create_run_layout`
- `write_run_state`
- `write_runtime_policy`
- `write_executor_descriptor`
- `write_checkpoint`
- `write_failure_record`
- `write_manifest`
- `write_run_manifest`
- `write_canonical_json`
- `prepare_tool_run_dirs`
- `build_telemetry_adapter`

## Ownership Rules
- CLI parsing belongs outside this crate.
- Tool selection and stage planning belong outside this crate.
- Backend process execution belongs to runner crates.
- Runtime may write declared run-layout artifacts through typed APIs only.

## Local Verification Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runtime --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runtime --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runtime --test schemas --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runtime --no-default-features
```

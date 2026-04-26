# Commands

`bijux-dna-runtime` is a library crate. It does not expose CLI commands or `src/bin` entrypoints, and it must not spawn backend commands.

## Runtime Commands
None.

## Managed Command Families
None.

## Runtime Entry Points
These are library functions, not shell commands:

- `create_run_layout`
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

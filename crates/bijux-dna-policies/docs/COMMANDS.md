# Commands

`bijux-dna-policies` is a policy library crate. It does not expose CLI commands, subcommands, runtime entrypoints, or tool execution wrappers.

## Runtime Commands
None.

## Managed Policy Commands
These repository commands are the policy gates this crate defines, documents, or validates:

- `make guardrails`
- `make policies`
- `make structure-check`

## Package Test Commands
Use these when changing this crate:

- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test boundaries`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test contracts`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test determinism`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test guardrails`

## Ownership Rules
- Keep runtime command execution in CLI, runner, runtime, or environment crates.
- Keep policy command documentation here and in `ENFORCEMENT.md`.
- Add new policy gate commands here before relying on them in crate docs or policy failure messages.

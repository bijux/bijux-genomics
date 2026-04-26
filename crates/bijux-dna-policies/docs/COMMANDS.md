# Commands

`bijux-dna-policies` is a policy library crate. It does not expose CLI commands, subcommands, runtime entrypoints, or tool execution wrappers.

## Runtime Commands
None.

## Managed Command Inventory

### Policy Gates
This crate defines package-level policy tests. Repository-level Make targets are owned by the
workspace Makefiles and `bijux-dna-dev`; do not list a Make target here unless that target exists
in the root Makefile stack and delegates to this package.

None.

## Package Test Commands
Use these when changing this crate:

- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test boundaries --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test contracts --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test determinism --no-default-features`
- `CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-policies --test guardrails --no-default-features`

## Ownership Rules
- Keep runtime command execution in CLI, runner, runtime, or environment crates.
- Keep policy command documentation here and in `ENFORCEMENT.md`.
- Add new policy gate commands here before relying on them in crate docs or policy failure messages.

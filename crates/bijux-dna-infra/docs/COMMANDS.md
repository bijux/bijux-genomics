# bijux-dna-infra Commands

`bijux-dna-infra` is library-only. It owns no Cargo binaries, CLI subcommands, host commands, or
process orchestration.

## Managed Command Inventory

None.

## Host Commands Managed By This Crate

None.

## Allowed Operations

The crate may provide generic filesystem, hashing, locking, retry, logging, path, temp-directory,
and config-format helpers to callers. Callers own any CLI command or host process that uses those
helpers.

## Forbidden Command Ownership

- No Cargo binary targets.
- No `std::process::Command` execution.
- No shell command wrappers.
- No Docker, Apptainer, Git, network-fetch, benchmark-runner, or workflow orchestration commands.

## Change Rules

Adding a command surface is a boundary change. Update this file, `BOUNDARY.md`, dependency tests,
and the crate architecture docs in the same change set before adding the code.

## Verification

Use:

```sh
TEST_TMP_DIR=artifacts/test-tmp CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-infra --no-default-features --test boundaries
```

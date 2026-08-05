# Commands

`bijux-dna-runner` is a library crate. It does not expose Cargo binary targets or a CLI surface.

## Runtime Commands
None.

## Managed Command Inventory

### Backend Commands
This crate may construct and execute only these backend command families from typed execution specs:

- `docker run`
- `apptainer exec`
- declared local commands through `LocalRunner`
- declared observer commands through `execute_observer_command`
- declared low-level commands through `run_command` and `run_command_with_context`
- non-waiting Slurm submission through `sbatch`
- one-shot Slurm queue observation through `squeue`
- terminal scheduler accounting through `sacct`
- explicit cancellation through `scancel`

`prepare_benchmark_slurm_job` consumes a validated immutable benchmark execution specification,
rechecks its persisted specification, image, and input identities, then publishes a restart-safe
Slurm script. It does not submit the script. The generated job runs Apptainer without network
access, mounts source and inputs read-only, records measured resource rows, and publishes completion
evidence only after required-output closure succeeds.

Generated benchmark jobs may invoke `git rev-parse`, `git status`, `sha256sum`, `grep`, `awk`, and
GNU `time` only for source validation, byte-identity checks, acceptance checks, and resource
evidence. Shell filesystem utilities are limited to the declared attempt result and scratch roots.

### Non-Executing Operations
Replay never executes a backend command. `replay_run` reads an execution manifest and verifies artifacts on disk.

Slurm operations never loop or wait for job completion. Callers persist the returned
job identity and invoke observation or collection in a later cycle.

## Ownership Rules
- CLI parsing stays in CLI/API crates.
- Tool selection and stage planning stay in planner/domain crates.
- Backend command construction must start from typed contracts, not ad hoc strings.
- Network access is disabled by default unless runtime policy explicitly declares it.

## Local Verification Commands

Run from the `bijux-genomics` repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runner --test boundaries --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runner --test contracts --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runner --test determinism --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-runner --no-default-features
```

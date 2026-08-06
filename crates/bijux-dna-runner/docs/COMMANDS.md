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

`submit_prepared_benchmark_job` persists an immutable submission intent before invoking `sbatch`.
It returns an existing matching handle without resubmission and refuses an intent with no handle so
an interrupted scheduler effect cannot silently create a duplicate immutable attempt.

`load_benchmark_slurm_job_handle` verifies and loads retained submission evidence without requiring
benchmark outputs to remain empty, allowing later one-shot status and collection operations to stay
inside runner-owned evidence contracts.

`prepare_corpus_materialization_slurm_job` publishes an immutable specification and script that
bind the frozen repository commit, executable digest, acquisition-plan digest, data root, resource
policy, and admission timestamp. `submit_prepared_corpus_materialization_job` persists intent before
the scheduler effect. The observe operation is one-shot and non-mutating; terminal collection
re-hashes the admitted catalog closure and publishes immutable pass/fail evidence.

Generated benchmark jobs may invoke `git rev-parse`, `git status`, `sha256sum`, `grep`, and `awk`
only for source validation, byte-identity checks, and acceptance checks. The Slurm host requires
GNU `time` at `/usr/bin/time` and wraps the complete offline `apptainer exec` to record resource
evidence without adding a timing-package dependency to scientific tool images. Shell filesystem
utilities are limited to the declared attempt result and scratch roots.

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

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

`prepare_managed_slurm_job` binds a clean repository commit, exact executable worker, attempt root,
resource request, dependency condition, and submission timestamp into immutable scheduler material.
`submit_prepared_managed_slurm_job` persists intent before `sbatch`; observation is one-shot, and
terminal collection admits scheduler accounting plus exact stdout and stderr identities. The worker
owns no scheduler calls, and the runner does not interpret its product outputs.

`prepare_image_campaign_slurm_job` binds the permanent `run-1000` protocol, frozen repository and
operator identities, exact Apptainer executable, canonical retained-tool closure, governed attempt
root, and bounded resources. Submission persists intent before `sbatch`; observation performs one
scheduler query; terminal collection binds accounting and logs, then delegates campaign receipt
admission to `bijux-dna-containers`. Repeated collection returns the original immutable evidence
when scheduler accounting and log identities remain unchanged.

`load_benchmark_slurm_job_handle` verifies and loads retained submission evidence without requiring
benchmark outputs to remain empty, allowing later one-shot status and collection operations to stay
inside runner-owned evidence contracts.

`prepare_corpus_materialization_slurm_job` publishes an immutable specification and script that
bind the frozen repository commit, executable digest, acquisition-plan digest, data root, resource
policy, and admission timestamp. It also persists `prepared-job.json`, allowing a separate process
to recover the exact submission request through `load_prepared_corpus_materialization_slurm_job`.
`submit_prepared_corpus_materialization_job` persists intent before the scheduler effect. The
observe operation is one-shot and non-mutating; terminal collection re-hashes the admitted catalog
closure and publishes immutable pass/fail evidence.

`prepare_container_qualification_slurm_job` requires the current qualification pipeline schema and
a commit-addressed frozen repository. It archives governance, pipeline, and credential-reference
contracts, binds their hashes plus the exact executor program and script, and publishes an
immutable scheduler specification. `submit_prepared_container_qualification_job` persists intent
before `sbatch`; observe and collect perform one scheduler query per call; terminal collection
binds accounting, logs, and successful execution evidence. Cancellation uses the retained exact
job handle. Failed current attempts can be archived and rolled back through the shared owner-scoped
attempt transaction; historical layouts remain read-only. None of these operations poll or wait.

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

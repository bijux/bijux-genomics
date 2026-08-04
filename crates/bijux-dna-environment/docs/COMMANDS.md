# bijux-dna-environment Commands

`bijux-dna-environment` is a library crate, not a CLI package. It has no `src/main.rs`, `src/bin`,
or Cargo `[[bin]]` target. The command inventory below is the SSOT for host commands that the
library can manage when callers invoke the matching API.

## Managed Command Inventory

- `available_runners()`: probes `docker --version`, `apptainer --version`, and
  `singularity --version`.
- `docker_image_exists()`: runs `docker image inspect <image>` for Docker images only.
- `run_shell_capture()`: runs `sh -lc <command>` and returns merged stdout/stderr for explicit
  caller-provided diagnostics.
- `run_smoke_script()`: runs `cargo run -q -p bijux-dna-dev -- containers run <smoke-command>` for
  one tool.
- `run_smoke_script_batch()`: runs the same developer-control-plane smoke command for a tool list
  and smoke level.
- `ReferenceRegistry::prepare_reference()`: when requested, runs `samtools faidx`, `gatk
  CreateSequenceDictionary`, `bwa index`, and `bowtie2-build` to materialize missing reference
  indexes.
- `verify_apptainer_tool_contract()`: runs `apptainer exec --cleanenv --containall` to read the
  embedded contract and `apptainer inspect --json` to compare its immutable labels.
- `build_apptainer_image()`: runs `apptainer build --force` with append-only attempt logs and
  retries only failures classified as transient OCI transport failures.
- `run_apptainer_smoke()`: runs offline `apptainer exec --containall --cleanenv --net --network
  none` probes and admits version output against the catalog contract regex.

## Non-Commands

- Image resolution and platform loading do not run containers.
- Catalog digest hydration reads the local registry file only; it does not contact a registry.
- Cache path functions compute paths; they do not pull, build, or delete images.

## Boundary Rules

- Additions to `std::process::Command` usage require this file and `tests/boundaries/commands.rs`
  to change in the same commit.
- Product execution belongs in runner/stage/API layers, not here.
- CLI command names belong in the CLI crate; this file lists library-managed host commands only.

## Process Ownership Files

- `src/resolve/commands.rs`: runner probes, Docker image inspection, and reference-index command
  execution helper.
- `src/resolve/shell.rs`: explicit shell capture for caller-provided diagnostics.
- `src/resolve/smoke.rs`: environment smoke handoff through the developer control plane.
- `src/evidence/tool_contract/inspection.rs`: embedded Apptainer contract and label inspection.
- `src/evidence/tool_contract/smoke.rs`: offline contract smoke execution and evidence admission.
- `src/build/apptainer.rs`: immutable Apptainer build attempts and bounded transport retries.

## Verification

Use:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-environment --no-default-features --test boundaries
```

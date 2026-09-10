# bijux-dna-containers Commands

`bijux-dna-containers` is a library crate, not a CLI package. It has no `src/main.rs`, `src/bin`,
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
- `publish_canonical_apptainer_image()`: owns the complete retained-image transaction: requires a
  clean Git commit, verifies the embedded contract and labels, and no-clobber publishes a
  digest-named SIF plus build evidence. The active destination uses `<study>/containers/apptainer`;
  the commit-campaign destination uses `<study>/containers/staging/<commit>` and
  `<study>/containers/evidence/image-campaigns/<commit>` without changing the active root. It runs
  `hostname` and `apptainer --version` to bind the builder environment to the evidence.
- `execute_apptainer_image_campaign()`: owns catalog-wide bounded workers, immutable planning,
  atomic per-tool progress, fresh resumed-image inspection, append-only invocation records, and
  all-passed terminal execution publication. An explicit retained-tool selection executes only
  that dependency closure under deterministic `selected-<tool-set-sha256>` staging and evidence
  roots; the complete retained roster preserves the historical commit-addressed root.
- `execute_apptainer_image_campaign_attempt()`: binds one campaign execution to an immutable
  attempt root, Slurm job identity, completion timestamp, execution digest, and atomic terminal
  receipt without owning scheduler submission or observation.
- `freeze_repository()`: runs bounded Git inspection and worktree commands to materialize an
  immutable commit-addressed checkout under site-governed roots.
- `archive_frozen_repository()` and `verify_frozen_repository_archive()`: create and verify a
  checksum-bound Git bundle from either an existing governed snapshot or the exact clean active
  repository when snapshot inode capacity is unavailable, then use an isolated bare proof
  repository to establish commit, sparse-index, cleanliness, and self-contained object identity
  before publication.
- `retire_frozen_repository()` and `restore_frozen_repository_archive()`: remove only the exact
  receipt-bound checkout after archive verification and atomically reconstruct the same read-only
  sparse checkout when rollback is required.
- `tracked_repository_files_under()`: runs bounded `git ls-files` inspection for one safe
  repository-relative prefix and returns sorted tracked file identities.
- `import_run_evidence()`, `plan_final_run_evidence()`, and
  `apply_final_run_evidence_plan()`: import immutable run locks, select the latest passing run per
  tool, and preserve superseded attempt lineage behind an explicit non-mutating plan boundary.
- `admit_external_artifact()`: explicitly admits an operator-provided artifact, records audited
  access denial, or records audited artifact absence. Admission runs the cataloged version probe
  against a binary or through `apptainer exec`, verifies the declared checksum, and publishes a
  no-copy record. Restricted artifacts must remain outside the repository and active image root;
  blocked and unavailable records remain product-supported but cannot bind an execution command.
- `admit_external_artifact_with_probe()`: applies the same admission rules through an explicit
  `ExternalArtifactProbeTransport`. A trusted transport supplies bounded raw observations;
  version matching and publication remain owned by admission. Transport unavailability is an error.
- `catalog_identity_at_commit()`: runs `git show --no-ext-diff <commit>:containers/contracts/catalog.toml`
  after strict commit validation and hashes the exact historical catalog bytes.
- `derive_container_catalog_lock()` and `load_container_catalog_lock()`: derive and admit the
  immutable lock only when the complete catalog and repository recipe closure validate.
- `derive_container_version_lock()`: runs `git log -1 --format=%cI` for the governed version
  source when no current lock date can be retained, then binds source, generator, and available
  runtime evidence into the immutable version lock.
- `apptainer_recipe_identity_at_commit()`: runs `git show --no-ext-diff` for one definition and
  `git rev-parse --verify` for each repository-owned `%files` source at an exact commit.
- `repository_head_commit()`: runs `git rev-parse --verify HEAD^{commit}` and rejects abbreviated or
  malformed identities.
- `clean_repository_head_commit()`: runs `git status --porcelain=v1 --untracked-files=normal`
  before resolving `HEAD` and rejects uncommitted build inputs.
- `admit_container_pipeline_site()`: runs bounded `git rev-parse --verify HEAD^{commit}` and
  `git status --porcelain=v1 --untracked-files=normal` inspections before admitting a pipeline's
  repository, immutable files, governed result layout, and Slurm resource selection.

## Non-Commands

- Image resolution and platform loading do not run containers.
- Catalog digest hydration reads the local registry file only; it does not contact a registry.
- Cache path functions compute paths; they do not pull, build, or delete images.
- Runtime and stage probes are executed and published by `bijux-dna-containers-qa`; this crate
  supplies catalog, artifact, recipe, and image identity contracts.

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
- `src/evidence/tool_contract/catalog.rs`: exact-commit catalog materialization for evidence identity.
- `src/evidence/tool_contract/recipe_identity.rs`: exact-commit definition and `%files` identity.
- `src/evidence/tool_contract/version_lock.rs`: committed version-source timestamp derivation for
  deterministic immutable lock publication.
- `src/evidence/external_artifact/probe/native.rs`: local operator-artifact version observation
  through shared process capture, with a 30-second deadline and 64 KiB combined raw stream limit.
- `src/site_policy/repository_freeze.rs`: clean-source verification and commit-addressed Git
  worktree materialization under site-governed roots.
- `src/site_policy/repository_archive.rs`: bounded Git-bundle creation from an existing snapshot or
  exact clean active source, empty-bare-repository completeness proof, receipt-gated checkout
  retirement, and atomic rollback under the same governed root.
- `src/site_policy/repository_tracking.rs`: prefix-scoped tracked-file discovery for governed
  repository publications.
- `src/execution_contract/pipeline/admission.rs`: execution-site repository identity and clean-state
  inspection before immutable pipeline inputs are admitted.
- `src/build/apptainer.rs`: immutable Apptainer build attempts and bounded transport retries.
- `src/build/publication.rs`: builder host, Apptainer version, pinned base, embedded identity, and
  canonical image publication.

## Verification

Use:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-containers --no-default-features --test boundaries
```

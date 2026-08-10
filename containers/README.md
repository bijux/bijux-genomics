# Containers

Authoritative entrypoint:
- [containers/index.md](index.md)

Operational docs index:
- [containers/docs/index.md](docs/index.md)
- [containers/docs/LUNARC_EXECUTION_CONTRACT.md](docs/LUNARC_EXECUTION_CONTRACT.md)
- [containers/reports/README.md](reports/README.md)
- [scientific benchmark authority](../benchmarks/README.md)
- [containers/reports/lunarc-fastq-bam.md](reports/lunarc-fastq-bam.md)
- [containers/reports/lunarc-vcf-phasing.md](reports/lunarc-vcf-phasing.md)
- [containers/reports/lunarc-vcf-relatedness.md](reports/lunarc-vcf-relatedness.md)
- [containers/reports/lunarc-reference-complexity.md](reports/lunarc-reference-complexity.md)
- [containers/reports/lunarc-amplicon-asv.md](reports/lunarc-amplicon-asv.md)
- [containers/docs/EVIDENCE_QUALIFICATION.md](docs/EVIDENCE_QUALIFICATION.md)
- [containers/docs/VERSION_AUTHORITY.md](docs/VERSION_AUTHORITY.md)
- [containers/docs/SCIENCE_EVIDENCE_BOUNDARY.md](docs/SCIENCE_EVIDENCE_BOUNDARY.md)
- [containers/licenses/README.md](licenses/README.md)
- [containers/docs/GHCR_PUBLISH.md](docs/GHCR_PUBLISH.md) (GHCR publication surface)
- `https://github.com/bijux?tab=packages&repo_name=bijux-genomics` (repository packages view)
- [containers/docs/SMOKE_CONTRACT.md](docs/SMOKE_CONTRACT.md) (smoke contract)

Site paths and scheduler policy are configuration, not source-code constants.
The reviewed LUNARC authority is
[`containers/contracts/sites/lunarc.yml`](contracts/sites/lunarc.yml); every
submitted run archives its resolved copy alongside the pipeline and redacted
credential-reference contracts.

## Build identity-governed LUNARC images

Build or reuse Apptainer images only through the Rust environment service. It
derives each build identity from the clean repository commit, catalog, recipe,
source pin, architecture, and declared interface. The generated definition
installs exactly one `/usr/share/bijux/tool-contract.json` and matching labels;
the resulting SIF is rejected before reuse when its digest, embedded contract,
or labels differ.

Run from a clean checkout at the commit being qualified. `--hpc-root` is the
exact study root that owns canonical images and evidence. Launch the retained
roster through one resumable campaign owner:

```console
cargo run -p bijux-dna -- env ensure-images \
  --hpc-root /lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY \
  --all-retained \
  --campaign \
  --jobs 4 \
  --json
```

With `--all-retained --campaign`, the command validates the exact catalog roster,
writes an immutable plan, runs bounded independent workers, persists progress
after every tool, appends one immutable execution record per invocation, and
freshly re-verifies any resumed image before skipping it. Canonical
`execution.json` is published only when the complete retained roster passes.
It stages and publishes each digest-named SIF under
`STUDY/containers/staging/<commit>` and creates immutable build evidence under
`STUDY/containers/evidence/image-campaigns/<commit>`. It verifies the embedded
contract and labels before publication and never writes the legacy active image
root. Omitting `--campaign` retains the explicit active-root transaction used by
reviewed storage migration. An abandoned staging path, digest-prefix collision,
dirty source tree, changed source commit, or conflicting evidence fails closed
for reconciliation. Do not invoke retired Python image campaign or cache-oriented
compatibility commands to create canonical image state. Reports from those paths
are historical records, not current qualification evidence.

Image construction runs on the approved frontend. Scientific tool execution
remains a Slurm workload with separate pipeline, input, scheduler, runtime, and
scientific evidence contracts.

Derive scan evidence and its portable publication from a clean,
commit-addressed checkout. Both outputs are immutable and the command refuses
to replace either path:

```console
study_root="/lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY"
image_commit="${image_commit:?set exact image campaign commit}"
source_repository="${study_root}/artifacts/source-checkouts/bijux-genomics-${image_commit:0:10}"
image_root="${study_root}/containers/staging/${image_commit}"
build_evidence_root="${study_root}/containers/evidence/image-campaigns/${image_commit}"
run_evidence_root="${run_evidence_root:?set governed run-evidence root}"
attempt_id="${attempt_id:?set immutable attempt id}"
inventory_root="${study_root}/containers/evidence/inventory/${image_commit}/${attempt_id}"
mkdir -p "${inventory_root}"

cargo run -p bijux-dna-dev -- containers run generate-image-inventory -- \
  "${source_repository}" \
  "${study_root}" \
  "${image_root}" \
  "${build_evidence_root}" \
  "${run_evidence_root}" \
  "${inventory_root}/scan.json" \
  "${inventory_root}/image-inventory.json"
canonical_inventory="${inventory_root}/image-inventory.json"
```

The source checkout and all evidence roots are mandatory. This allows a newer
automation binary to inventory an older exact-commit image campaign without
misstating the campaign's repository identity or scanning an unrelated active
image tree. The raw scan persists the ordered portable image/build-evidence
root pairs, so every composite derivation can be reproduced without recovering
its command line from scheduler logs.

When a current single-tool campaign supersedes one image from an otherwise
compatible prior campaign, append its image root and matching build-evidence
root as an additional pair. The container owner scans every declared pair as
one evidence set, rejects duplicate roots and image digests, and applies the
current repository recipes to the combined candidates. Prior campaign roots
remain immutable; no copied or synthetic build evidence is admitted.

## Activate a reviewed image campaign

Activation is a separate no-clobber transaction. It selects the unique
canonical image for every retained tool from the immutable campaign inventory,
rechecks each source identity, and publishes same-filesystem hard links into
the active root without removing staging evidence. The plan persists progress
after every image and accepts an interrupted publication only when source and
target are the same inode.

```console
activation_root="${study_root}/containers/evidence/activation/${image_commit}/${attempt_id}"
activation_plan="${activation_root}/image-activation-plan.json"
mkdir -p "${activation_root}"

cargo run -p bijux-dna-dev -- containers run image-activation -- \
  plan "${study_root}" "${canonical_inventory}" "${image_root}" \
  "${study_root}/containers/apptainer" "${activation_plan}" \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cargo run -p bijux-dna-dev -- containers run image-activation -- \
  apply "${activation_plan}" "${image_root}" \
  "${study_root}/containers/apptainer" \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cargo run -p bijux-dna-dev -- containers run image-activation -- \
  verify "${activation_plan}" "${image_root}" \
  "${study_root}/containers/apptainer"
```

Review the complete preview before applying it. Rollback removes only the
verified active links recorded by the plan and leaves every staging source in
place.

## Reconcile canonical image storage

Storage reconciliation is a typed, reversible environment workflow. It accepts
inventory rows classified as historical run dependencies, unreferenced
obsolete, or interrupted,
rechecks every regular-file size and digest, and refuses targets outside a
same-filesystem quarantine beneath the image root. Planning and report
publication are no-clobber operations. Apply and rollback persist progress
after every rename and resume only when the discovered file matches the locked
identity. Historical dependencies leave the active top level but remain
checksummed under the `historical/` quarantine category; their run-reference
authority remains in the plan's immutable source inventory.

Preview the exact move set before applying it:

```console
quarantine_root="${study_root}/containers/apptainer/.quarantine/${image_commit}/${attempt_id}"
storage_plan="${inventory_root}/image-storage-plan.json"

cargo run -p bijux-dna-dev -- containers run image-storage -- \
  plan "${study_root}" "${canonical_inventory}" \
  "${study_root}/containers/apptainer" "${quarantine_root}" \
  "${storage_plan}" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

After review, apply and verify the plan. Generate a new inventory at the same
source and catalog identity, then derive the portable report from that exact
post-storage inventory:

```console
cargo run -p bijux-dna-dev -- containers run image-storage -- \
  apply "${storage_plan}" "${study_root}/containers/apptainer" \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

cargo run -p bijux-dna-dev -- containers run image-storage -- \
  verify "${storage_plan}" "${study_root}/containers/apptainer"

cargo run -p bijux-dna-dev -- containers run image-storage -- \
  report "${storage_plan}" "${inventory_root}/after-storage.json" \
  "${inventory_root}/image-storage.json" \
  "${inventory_root}/image-storage.md"
```

The report passes only when every repository-delivered image is current, every
external or site-delivered tool is explicitly not applicable to image storage,
and no unknown, staged, or unreferenced obsolete image remains. A failed report
is evidence of an incomplete roster, not permission to weaken the checks.
Until the plan is accepted, rollback is available with the same image root and
a new UTC observation timestamp.

## Verify version and help contracts

Image admission proves that a recipe produced an inspectable SIF; it does not
prove that the packaged entry points start correctly. After inventory and
quarantine are complete, run the catalog-wide runtime campaign on the approved
frontend. It executes version and help checks with a disabled runtime network,
an isolated resolvable hostname, bounded command time, and one immutable
stdout/stderr pair per check. The same hostname policy applies to governed
Slurm execution so hostname-sensitive runtimes behave consistently without
restoring network access:

```console
runtime_root="/lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY/containers/evidence/runtime/${image_commit}"
publication_root="${runtime_root}/publication"

cargo run -p bijux-dna-containers-qa --bin runtime_verification -- execute \
  "${source_repository}" \
  /lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY \
  "${canonical_inventory}" \
  "$(command -v apptainer)" \
  /lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY/data \
  "${runtime_root}" \
  "${publication_root}/runtime-verification.json" \
  "${publication_root}/runtime-verification.md" \
  8 \
  30
```

The QA crate resolves every image through the canonical inventory,
re-hashes it before execution, runs independent workers without polling or
serial queueing, and preserves one immutable manifest plus stdout/stderr pair
per probe. Every invocation re-inspects the embedded contract and image labels;
an existing identity artifact must equal that fresh observation and cannot
replace it. Existing matching smoke manifests are reused after interruption;
identity drift or changed bytes at an immutable path fail closed. The portable
publication binds each tool to its identity evidence and smoke manifest with
`study://` paths and exact SHA-256 identities.

A passing runtime campaign qualifies only the version and help dimensions.
Real one-sample execution remains a governed Slurm run, and scientific,
benchmark, security, and release claims remain independent. The explicit
source repository must be the clean checkout whose HEAD matches the inventory
commit; later movement of the automation checkout cannot silently change the
campaign.

## Govern qualification run identities

[`contracts/qualification-runs.toml`](contracts/qualification-runs.toml) is the
stable container run protocol registry. Its tool allocation map preserves
successful checked-in run IDs, allocates missing runnable identities strictly
above the historical maximum, and gives licensed external-artifact tools the
same permanent protocol identity and resource contract as packaged tools.
External-artifact protocols remain allocated, and execution reports
`blocked_external` until a checksum-bound binary or SIF admission is available;
the redistribution disposition never removes them from qualification. Its
semantic run definitions close every identity from `run-1000` through
`run-1158`, including the image campaign, historical protocols, current
protocols, allocated next actions, and permanent `run-1081`/`run-1085`
tombstones. Every definition carries purpose, ownership,
operation, tools, contracts, corpus roles, acceptance, supersession, and a
stable citation record. Catalog stage and input-role drift, cross-tool
historical ID reuse, duplicate final runs, and non-passing final evidence fail
closed.
[`contracts/qualification-run-history.toml`](contracts/qualification-run-history.toml)
preserves checksummed LUNARC identities that failed before an execution attempt
could receive a Slurm job ID.

Regenerate and review the registry together with its
[run-ID audit](reports/qualification-run-id-audit.json):

```console
cargo run -p bijux-dna-dev -- containers run qualification-registry -- \
  generate containers/runs containers/runs/attempt-ledger.json \
  containers/contracts/qualification-run-history.toml \
  containers/contracts/qualification-runs.toml \
  containers/reports/qualification-run-id-audit.json

cargo run -p bijux-dna-dev -- containers run qualification-registry -- \
  check containers/runs containers/runs/attempt-ledger.json \
  containers/contracts/qualification-run-history.toml \
  containers/contracts/qualification-runs.toml \
  containers/reports/qualification-run-id-audit.json
```

This registry owns protocol identity and citation metadata only. It does not
make historical evidence current, generate execution specifications, submit
Slurm jobs, or qualify a tool; those actions require the separate attempt,
specification, publication, and lifecycle gates.

## Prepare governed input data

Container validation does not download or silently replace scientific input.
Admit a file beneath the configured study `data/` root, verify its format with
the domain-appropriate reader, and record its SHA-256 in `config-pipeline.yml`.
For the paired FASTQ corpus used by the current real-run examples:

```console
data_root="/lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY/data/corpora/ena-prjeb10854-err1021817"
gzip -t "${data_root}/ERR1021817.read-prefix-100000_1.fastq.gz"
gzip -t "${data_root}/ERR1021817.read-prefix-100000_2.fastq.gz"
sha256sum "${data_root}/ERR1021817.read-prefix-100000_1.fastq.gz" \
  "${data_root}/ERR1021817.read-prefix-100000_2.fastq.gz"
```

The validator re-hashes every declared input, image, and frozen repository
before submission. A missing file or checksum mismatch is a hard failure; the
executor never substitutes a fixture.

Scientific data selection and admission belong to `bijux-dna-bench`, not to
container runtime contracts. Container runs consume its verified index
instead of encoding knowledge of individual catalog shapes:

```console
study=/lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY
evidence="$study/data/evidence/indexes/$(git rev-parse HEAD)"
cargo run -q -p bijux-dna -- bench data-index \
  --catalog-root "$study/data/catalog" \
  --data-root "$study/data" \
  --output "$evidence/data-index.json" \
  --report "$evidence/data-index.md" \
  --created-at-utc "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

The benchmark-owned index recalculates every governed file SHA-256, replaces execution-site
absolute paths with `study://data/...` identities, and reports missing source
or temporal provenance without inventing values. A record is benchmark-eligible
only when its governed files, source identity, retrieval or admission time,
access classification, and limitations are explicit.

## Prepare and submit a real run

Draft specifications are staged under the governed study evidence root. The
repository commit named by the specification must first be frozen into a clean
execution checkout:

```console
run_commit="$(git rev-parse HEAD)"
cargo run -q -p bijux-dna-dev -- containers run repository-freeze -- \
  --governance containers/contracts/sites/lunarc.yml \
  --commit "${run_commit}" \
  --scope complete
cargo build --locked --release -p bijux-dna-dev
operator_program="$(pwd)/artifacts/rust/target/release/bijux-dna-dev"
test -x "${operator_program}"

run_spec="/lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY/containers/evidence/specifications/run-1125-trimmomatic"
attempt_id=attempt-001
cargo run -q -p bijux-dna-dev -- containers run qualification-slurm -- prepare \
  "${attempt_id}" \
  containers/contracts/sites/lunarc.yml \
  "${run_spec}/config-pipeline.yml" \
  "${run_spec}/config-credential.yml" \
  "${operator_program}" \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)"

qualification_spec="/lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY/containers/evidence/submissions/run-1125-trimmomatic/attempts/${attempt_id}/qualification-spec.json"
cargo run -q -p bijux-dna-dev -- containers run qualification-slurm -- submit \
  "${qualification_spec}"
```

To validate every staged specification before submitting any of them, run:

```console
spec_root="/lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY/containers/evidence/specifications"
for run_spec in "${spec_root}"/run-*-*; do
  cargo run -q -p bijux-dna-dev -- containers run credential-reference -- validate \
    "${run_spec}/config-credential.yml"
  cargo run -q -p bijux-dna-dev -- containers run pipeline-contract -- admit \
    --governance containers/contracts/sites/lunarc.yml \
    "${run_spec}/config-pipeline.yml"
done
```

Submission remains an explicit per-run action so a malformed or scientifically
inappropriate batch cannot enter the queue merely because it shares a folder.
Each preparation requires a nonzero `attempt-NNN` identity. Submission evidence
and result data are isolated below that attempt, so a retry cannot overwrite or
invalidate an earlier attempt.

## Collect results and publish run evidence

After Slurm reaches a terminal state, collection adds immutable scheduler
accounting and preserves failures even when the executor never started:

```console
cargo run -q -p bijux-dna-dev -- containers run qualification-slurm -- collect \
  "${qualification_spec}" \
  "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

If collected scheduler evidence marks the attempt failed, archive it before
correcting and resubmitting the same stable run specification. This also covers
scheduler failures that occurred before execution could publish `evidence.json`:

```console
cargo run -q -p bijux-dna-dev -- containers run qualification-slurm -- archive-failure \
  "${qualification_spec}" "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
```

Archival accepts only failed terminal evidence with matching Slurm accounting.
It hashes every submission and result file, verifies that both trees and the
owner-specific archive root share one filesystem, then moves them through a
recoverable partial directory. Qualification failures remain under
`containers/evidence/failures/RUN-TOOL-slurm-JOB/`. This frees the canonical
submission and result paths without deleting the failed attempt or allowing it
into the final run index. A passed result and a benchmark attempt cannot be
archived by this qualification command.

If a passed attempt later proves scientifically or structurally invalid, keep
its submission and result trees immutable. Do not relabel it as failed or
reuse its attempt identity. Record supersession through the run lifecycle and
prepare the next `attempt-NNN`; historical import and replay retain the prior
attempt as evidence rather than executing a legacy worker.

Import one completed run by bounding the identifier. For a full replay, use an
empty artifact directory so existing tracked evidence is never overwritten:

```console
result_root="/lunarc/nobackup/projects/snic2019-34-3/Dardel/STUDY/results/container-validation"

# One run
cargo run -q -p bijux-dna-dev -- containers run run-evidence -- \
  import "${result_root}" containers/runs 1125 1125

# Full replay into disposable review output
replay_root="artifacts/containers/run-evidence-replay"
cargo run -q -p bijux-dna-dev -- containers run run-evidence -- \
  import "${result_root}" "${replay_root}"

cargo run -q -p bijux-dna-dev -- containers run run-evidence -- \
  publish "${replay_root}" "${replay_root}/attempt-ledger.json" --apply

# Publish the newly imported tracked run only after previewing the selection
cargo run -q -p bijux-dna-dev -- containers run run-evidence -- \
  publish containers/runs containers/runs/attempt-ledger.json
cargo run -q -p bijux-dna-dev -- containers run run-evidence -- \
  publish containers/runs containers/runs/attempt-ledger.json --apply

cargo run -q -p bijux-dna-dev -- containers run run-evidence -- \
  validate containers/runs
cargo run -q -p bijux-dna-dev -- containers run run-evidence -- \
  generate containers/runs \
  containers/reports/run-evidence.json \
  containers/reports/run-evidence.md
```

The all-runs importer is intentionally non-overwriting. Use an empty artifact
root for complete replay and comparison; use run bounds when adding final
evidence to the repository incrementally.

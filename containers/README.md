# Containers

Authoritative entrypoint:
- [containers/index.md](index.md)

Operational docs index:
- [containers/docs/index.md](docs/index.md)
- [containers/docs/LUNARC_EXECUTION_CONTRACT.md](docs/LUNARC_EXECUTION_CONTRACT.md)
- [containers/reports/README.md](reports/README.md)
- [containers/benchmarks/README.md](benchmarks/README.md)
- [containers/reports/stage-benchmark-coverage.md](reports/stage-benchmark-coverage.md)
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

## Reproduce the LUNARC image roster

`run-1000` is the governed frontend campaign that rebuilds every supported
Apptainer image. It is intentionally complete: a successful execution against
an empty admission root proves that the whole roster can be recovered from the
catalog, recipes, and pinned sources. The operational execution record resolves
the exact clean repository commit, catalog hash, inventory hash, admission
root, recipe hashes, logs, and admitted image hashes; the source campaign
cannot contain its own Git commit because a Git object cannot self-identify.

Inventory and campaign records are immutable and commit-addressed. Preparation
creates a clean, complete build worktree beneath the configured
`paths.frozen_repositories/image-builds/` root. The campaign reads recipes and
all `%files` inputs from that worktree, so later movement of the active checkout
cannot change an in-flight build. A routine proof uses an isolated root below
the configured `containers_root`; it never deletes or overwrites admitted
images. The same commands repopulate the canonical `apptainer/` directory after
an explicitly reviewed storage migration by setting `rebuild_root` to the
configured containers root.

```console
image_commit="$(git rev-parse HEAD)"
containers_root="/home/bijan/lu2024-12-24/Dardel/STUDY/containers"
rebuild_root="${containers_root}/rebuild-verification/${image_commit}"
inventory_root="${rebuild_root}/inventory"
campaign_root="${rebuild_root}/campaign"
mkdir -p "${rebuild_root}/apptainer" "${rebuild_root}/evidence" \
  "${inventory_root}" "${campaign_root}"

python3 -B containers/contracts/image_inventory.py audit \
  --catalog containers/contracts/catalog.toml \
  --image-root "${rebuild_root}/apptainer" \
  --evidence-root "${rebuild_root}/evidence" \
  --runs-root containers/runs \
  --repository . \
  --output "${inventory_root}/before-build.json"

python3 -B containers/contracts/image_campaign.py prepare \
  --campaign containers/runs/run-1000-images/config-campaign.json \
  --inventory "${inventory_root}/before-build.json" \
  --catalog containers/contracts/catalog.toml \
  --governance containers/contracts/sites/lunarc.yml \
  --image-root "${rebuild_root}/apptainer" \
  --output "${campaign_root}/execution.json"

python3 -B containers/contracts/image_campaign.py execute \
  --execution "${campaign_root}/execution.json" \
  --governance containers/contracts/sites/lunarc.yml \
  --jobs 2

# A transient build failure is retained as an attempt before only that tool is
# queued and executed again. Omit --tool to queue every failed tool.
python3 -B containers/contracts/image_campaign.py retry-failed \
  --execution "${campaign_root}/execution.json" \
  --tool addeam
python3 -B containers/contracts/image_campaign.py execute \
  --execution "${campaign_root}/execution.json" \
  --governance containers/contracts/sites/lunarc.yml \
  --jobs 1

frozen_repository="$(python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["frozen_repository"])' "${campaign_root}/execution.json")"

python3 -B containers/contracts/image_inventory.py audit \
  --catalog "${frozen_repository}/containers/contracts/catalog.toml" \
  --image-root "${rebuild_root}/apptainer" \
  --evidence-root "${rebuild_root}/evidence" \
  --runs-root "${frozen_repository}/containers/runs" \
  --repository "${frozen_repository}" \
  --output "${inventory_root}/after-build.json"

python3 -B containers/contracts/image_campaign.py report \
  --execution "${campaign_root}/execution.json" \
  --inventory "${inventory_root}/after-build.json" \
  --json-output containers/reports/lunarc-image-rebuild.json \
  --markdown-output containers/reports/lunarc-image-rebuild.md
```

The tracked [complete-roster rebuild report](reports/lunarc-image-rebuild.md)
is the reviewed source/build/freshness summary for this campaign.

The before-build inventory must report every catalog-supported image as missing
and preserve each unsupported disposition. The execution and after-build
inventory must report every selected image as passed/current with no staged or
unknown images. A retry preserves each failed attempt, its immutable build log,
and the successful final attempt in the same execution record; a campaign with
any failed tool remains failed even when there is nothing pending. Image
construction runs on the approved frontend. Real
scientific tool runs remain Slurm jobs with separate pipeline and
credential-reference contracts.

Superseded images and interrupted builds are never deleted from an ad hoc
shell command. After the image campaign is idle, create and inspect a
checksummed quarantine plan, then apply and verify its same-filesystem moves:

```console
canonical_inventory="${containers_root}/evidence/inventory/${image_commit}/image-inventory.json"

python3 -B containers/contracts/image_storage.py plan \
  --inventory "${canonical_inventory}" \
  --image-root "${containers_root}/apptainer" \
  --quarantine-root "${containers_root}/apptainer/.quarantine/${image_commit}" \
  --plan "${containers_root}/evidence/inventory/${image_commit}/image-storage-plan.json"

python3 -B containers/contracts/image_storage.py apply \
  --image-root "${containers_root}/apptainer" \
  --plan "${containers_root}/evidence/inventory/${image_commit}/image-storage-plan.json"

python3 -B containers/contracts/image_storage.py verify \
  --image-root "${containers_root}/apptainer" \
  --plan "${containers_root}/evidence/inventory/${image_commit}/image-storage-plan.json"

after_storage="${containers_root}/evidence/inventory/${image_commit}/after-storage.json"

python3 -B containers/contracts/image_inventory.py audit \
  --catalog containers/contracts/catalog.toml \
  --image-root "${containers_root}/apptainer" \
  --evidence-root "${containers_root}/evidence" \
  --runs-root containers/runs \
  --repository . \
  --output "${after_storage}"

python3 -B containers/contracts/image_storage.py report \
  --plan "${containers_root}/evidence/inventory/${image_commit}/image-storage-plan.json" \
  --after-inventory "${after_storage}" \
  --json-output "${containers_root}/evidence/inventory/${image_commit}/image-storage-report.json" \
  --markdown-output "${containers_root}/evidence/inventory/${image_commit}/image-storage-report.md"
```

`rollback` restores every file only after rechecking its size and SHA-256.
Unclassified images and images referenced by historical runs are excluded from
the quarantine plan. Catalog-recognized obsolete images with no run references
are eligible only through the checksummed, reversible plan above.
The tracked [canonical storage report](reports/image-storage.md) records the
reviewed move set, post-storage inventory, raw evidence digests, and recovery
contract without publishing user-specific absolute paths. Import those reviewed
report files into `containers/reports/` from a clean local checkout; do not
generate tracked outputs in the LUNARC checkout.

## Verify version and help contracts

Image admission proves that a recipe produced an inspectable SIF; it does not
prove that the packaged entry points start correctly. After inventory and
quarantine are complete, run the catalog-wide runtime campaign on the approved
frontend. It executes version and help checks with a disabled runtime network,
bounded command time, and one immutable stdout/stderr pair per check:

```console
runtime_root="/home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/runtime/${image_commit}"
mkdir -p "${runtime_root}"

python3 -B containers/contracts/runtime_verification.py prepare \
  --inventory "${canonical_inventory}" \
  --catalog containers/contracts/catalog.toml \
  --governance containers/contracts/sites/lunarc.yml \
  --repository "${frozen_repository}" \
  --output "${runtime_root}/execution.json"

python3 -B containers/contracts/runtime_verification.py execute \
  --execution "${runtime_root}/execution.json" \
  --governance containers/contracts/sites/lunarc.yml \
  --jobs 8 \
  --timeout-seconds 30
```

The execution record retains the exact frozen repository, catalog, inventory,
image, command, output, and log digests. A passing runtime campaign qualifies
only the version and help dimensions. Real one-sample execution remains a
governed Slurm run, and scientific, benchmark, security, and release claims
remain independent. `--repository` must identify a clean checkout beneath the
governed commit-addressed worktree root whose HEAD matches the inventory
commit; later movement of the active checkout cannot invalidate or silently
change this campaign.

Raw execution JSON keeps the absolute paths needed to audit files at LUNARC;
it is not suitable for a public repository. Publish portable companions before
reviewing or importing tracked reports:

```console
publication_root="${runtime_root}/publication"
mkdir -p "${publication_root}"

python3 -B containers/contracts/image_inventory.py publish \
  --inventory "${canonical_inventory}" \
  --study-root /home/bijan/lu2024-12-24/Dardel/STUDY \
  --repository "$(pwd)" \
  --output "${publication_root}/image-inventory.json"

python3 -B containers/contracts/runtime_verification.py publish \
  --execution "${runtime_root}/execution.json" \
  --study-root /home/bijan/lu2024-12-24/Dardel/STUDY \
  --repository "$(pwd)" \
  --output "${publication_root}/runtime-verification.json"
```

Publication replaces governed absolute roots with `study://` and
`repository://` identifiers and records the SHA-256 of the untouched raw JSON.
It fails when any absolute path lies outside those roots. Qualification accepts
only these portable publication schemas; execution and replay continue to use
the raw site records.

## Prepare governed input data

Container validation does not download or silently replace scientific input.
Admit a file beneath the configured study `data/` root, verify its format with
the domain-appropriate reader, and record its SHA-256 in `config-pipeline.yml`.
For the paired FASTQ corpus used by the current real-run examples:

```console
data_root="/home/bijan/lu2024-12-24/Dardel/STUDY/data/corpora/ena-prjeb10854-err1021817"
gzip -t "${data_root}/ERR1021817.read-prefix-100000_1.fastq.gz"
gzip -t "${data_root}/ERR1021817.read-prefix-100000_2.fastq.gz"
sha256sum "${data_root}/ERR1021817.read-prefix-100000_1.fastq.gz" \
  "${data_root}/ERR1021817.read-prefix-100000_2.fastq.gz"
```

The validator re-hashes every declared input, image, and frozen repository
before submission. A missing file or checksum mismatch is a hard failure; the
executor never substitutes a fixture.

## Prepare and submit a real run

Draft specifications are staged under the governed study evidence root. The
repository commit named by the specification must first be frozen into a clean
execution checkout:

```console
run_commit="$(git rev-parse HEAD)"
python3 -B containers/contracts/container_validation.py freeze-repository \
  --governance containers/contracts/sites/lunarc.yml \
  --commit "${run_commit}"

run_spec="/home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/specifications/run-1125-trimmomatic"
python3 -B containers/contracts/container_validation.py validate \
  --governance containers/contracts/sites/lunarc.yml \
  --pipeline "${run_spec}/config-pipeline.yml" \
  --credentials "${run_spec}/config-credential.yml"
python3 -B containers/contracts/container_validation.py submit \
  --manifest "${run_spec}" \
  --governance containers/contracts/sites/lunarc.yml
```

To validate every staged specification before submitting any of them, run:

```console
spec_root="/home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/specifications"
for run_spec in "${spec_root}"/run-*-*; do
  python3 -B containers/contracts/container_validation.py validate \
    --governance containers/contracts/sites/lunarc.yml \
    --pipeline "${run_spec}/config-pipeline.yml" \
    --credentials "${run_spec}/config-credential.yml"
done
```

Submission remains an explicit per-run action so a malformed or scientifically
inappropriate batch cannot enter the queue merely because it shares a folder.

## Collect results and publish run evidence

After Slurm reaches a terminal state, collection adds immutable scheduler
accounting and preserves failures even when the executor never started:

```console
python3 -B containers/contracts/container_validation.py collect-accounting \
  --manifest "${run_spec}" \
  --governance containers/contracts/sites/lunarc.yml
```

If the collected `evidence.json` is failed, archive that attempt before
correcting and resubmitting the same stable run specification:

```console
python3 -B containers/contracts/container_validation.py archive-failure \
  --manifest "${run_spec}" \
  --governance containers/contracts/sites/lunarc.yml
```

Archival accepts only failed terminal evidence with matching Slurm accounting.
It hashes every submission and result file, verifies that both trees and the
failure root share one filesystem, then moves them through a recoverable
partial directory into
`containers/evidence/failures/RUN-TOOL-slurm-JOB/`. This frees the canonical
submission and result paths without deleting the failed attempt or allowing it
into the final run index. A passed result cannot be archived by this command.

Import one completed run by bounding the identifier. For a full replay, use an
empty artifact directory so existing tracked evidence is never overwritten:

```console
result_root="/home/bijan/lu2024-12-24/Dardel/STUDY/results/container-validation"

# One run
python3 -B containers/runs/manage.py import-evidence \
  --result-root "${result_root}" \
  --output-root containers/runs \
  --minimum-run 1125 \
  --maximum-run 1125

# Full replay into disposable review output
replay_root="artifacts/containers/run-evidence-replay"
python3 -B containers/runs/manage.py import-evidence \
  --result-root "${result_root}" \
  --output-root "${replay_root}"

python3 -B containers/runs/manage.py publish-final-evidence \
  --runs-root "${replay_root}" \
  --ledger "${replay_root}/attempt-ledger.json" \
  --apply

# Publish the newly imported tracked run only after previewing the selection
python3 -B containers/runs/manage.py publish-final-evidence \
  --runs-root containers/runs \
  --ledger containers/runs/attempt-ledger.json
python3 -B containers/runs/manage.py publish-final-evidence \
  --runs-root containers/runs \
  --ledger containers/runs/attempt-ledger.json \
  --apply

python3 -B containers/runs/manage.py validate --runs-root containers/runs
python3 -B containers/runs/manage.py render-index \
  --runs-root containers/runs \
  --output containers/reports/run-evidence.json
python3 -B containers/runs/manage.py render-report \
  --runs-root containers/runs \
  --output containers/reports/run-evidence.md
```

The all-runs importer is intentionally non-overwriting. Use an empty artifact
root for complete replay and comparison; use run bounds when adding final
evidence to the repository incrementally.

# Containers

Authoritative entrypoint:
- [containers/index.md](index.md)

Operational docs index:
- [containers/docs/index.md](docs/index.md)
- [containers/docs/LUNARC_EXECUTION_CONTRACT.md](docs/LUNARC_EXECUTION_CONTRACT.md)
- [containers/reports/README.md](reports/README.md)
- [containers/reports/lunarc-fastq-bam.md](reports/lunarc-fastq-bam.md)
- [containers/reports/lunarc-vcf-phasing.md](reports/lunarc-vcf-phasing.md)
- [containers/reports/lunarc-vcf-relatedness.md](reports/lunarc-vcf-relatedness.md)
- [containers/reports/lunarc-reference-complexity.md](reports/lunarc-reference-complexity.md)
- [containers/reports/lunarc-amplicon-asv.md](reports/lunarc-amplicon-asv.md)
- [containers/docs/TOOL_LIFECYCLE.md](docs/TOOL_LIFECYCLE.md)
- [containers/docs/VERSION_AUTHORITY.md](docs/VERSION_AUTHORITY.md)
- [containers/docs/SCIENCE_EVIDENCE_BOUNDARY.md](docs/SCIENCE_EVIDENCE_BOUNDARY.md)
- [containers/licenses/README.md](licenses/README.md)
- [containers/docs/GHCR_PUBLISH.md](docs/GHCR_PUBLISH.md) (GHCR publication surface)
- `https://github.com/bijux?tab=packages&repo_name=bijux-genomics` (repository packages view)
- [containers/docs/SMOKE_CONTRACT.md](docs/SMOKE_CONTRACT.md) (smoke contract)
- [containers/docs/PROMOTION_POLICY.md](docs/PROMOTION_POLICY.md) (promotion and demotion policy)

Site paths and scheduler policy are configuration, not source-code constants.
The reviewed LUNARC authority is
[`containers/contracts/sites/lunarc.yml`](contracts/sites/lunarc.yml); every
submitted run archives its resolved copy alongside the pipeline and redacted
credential-reference contracts.

## Reproduce the LUNARC image roster

`run-1000` is the governed frontend campaign that prepares immutable SIFs. It
selects only supported catalog tools whose admitted image is missing or no
longer matches the active recipe. The operational execution record resolves
the exact clean repository commit, catalog hash, inventory hash, recipe hashes,
logs, and admitted image hashes; the source campaign cannot contain its own Git
commit because a Git object cannot self-identify.

Inventory and campaign records are immutable and commit-addressed. Preparation
creates a clean, complete build worktree beneath the configured
`paths.frozen_repositories/image-builds/` root. The campaign reads recipes and
all `%files` inputs from that worktree, so later movement of the active checkout
cannot change an in-flight build.

```console
image_commit="$(git rev-parse HEAD)"
inventory_root="/home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/inventory/${image_commit}"
campaign_root="/home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/campaigns/run-1000-images/${image_commit}"
mkdir -p "${inventory_root}" "${campaign_root}"

python3 containers/contracts/image_inventory.py audit \
  --catalog containers/contracts/catalog.toml \
  --image-root /home/bijan/lu2024-12-24/Dardel/STUDY/containers/apptainer \
  --evidence-root /home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence \
  --runs-root containers/runs \
  --repository . \
  --output "${inventory_root}/image-inventory.json"

python3 containers/contracts/image_campaign.py prepare \
  --campaign containers/runs/run-1000-images/config-campaign.json \
  --inventory "${inventory_root}/image-inventory.json" \
  --catalog containers/contracts/catalog.toml \
  --governance containers/contracts/sites/lunarc.yml \
  --output "${campaign_root}/execution.json"

python3 containers/contracts/image_campaign.py execute \
  --execution "${campaign_root}/execution.json" \
  --governance containers/contracts/sites/lunarc.yml \
  --jobs 2
```

Image construction runs on the approved frontend. Real scientific tool runs
remain Slurm jobs with separate pipeline and credential-reference contracts.

Superseded images and interrupted builds are never deleted from an ad hoc
shell command. After the image campaign is idle, create and inspect a
checksummed quarantine plan, then apply and verify its same-filesystem moves:

```console
python3 containers/contracts/image_storage.py plan \
  --inventory "${inventory_root}/image-inventory.json" \
  --image-root /home/bijan/lu2024-12-24/Dardel/STUDY/containers/apptainer \
  --quarantine-root "/home/bijan/lu2024-12-24/Dardel/STUDY/containers/apptainer/.quarantine/${image_commit}" \
  --plan "${inventory_root}/image-storage-plan.json"

python3 containers/contracts/image_storage.py apply \
  --image-root /home/bijan/lu2024-12-24/Dardel/STUDY/containers/apptainer \
  --plan "${inventory_root}/image-storage-plan.json"

python3 containers/contracts/image_storage.py verify \
  --image-root /home/bijan/lu2024-12-24/Dardel/STUDY/containers/apptainer \
  --plan "${inventory_root}/image-storage-plan.json"
```

`rollback` restores every file only after rechecking its size and SHA-256.
Unknown images and images referenced by historical runs are excluded from the
quarantine plan.

## Verify version and help contracts

Image admission proves that a recipe produced an inspectable SIF; it does not
prove that the packaged entry points start correctly. After inventory and
quarantine are complete, run the catalog-wide runtime campaign on the approved
frontend. It executes version and help checks with a disabled runtime network,
bounded command time, and one immutable stdout/stderr pair per check:

```console
runtime_root="/home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/runtime/${image_commit}"
mkdir -p "${runtime_root}"

python3 containers/contracts/runtime_verification.py prepare \
  --inventory "${inventory_root}/image-inventory-post-quarantine.json" \
  --catalog containers/contracts/catalog.toml \
  --governance containers/contracts/sites/lunarc.yml \
  --output "${runtime_root}/execution.json"

python3 containers/contracts/runtime_verification.py execute \
  --execution "${runtime_root}/execution.json" \
  --governance containers/contracts/sites/lunarc.yml \
  --jobs 8 \
  --timeout-seconds 30
```

The execution record retains the exact repository, catalog, inventory, image,
command, output, and log digests. A passing runtime campaign qualifies only the
version and help dimensions. Real one-sample execution remains a governed
Slurm run, and scientific, benchmark, security, and release claims remain
independent.

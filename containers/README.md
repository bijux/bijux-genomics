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

```console
python3 containers/contracts/image_inventory.py audit \
  --catalog containers/contracts/catalog.toml \
  --image-root /home/bijan/lu2024-12-24/Dardel/STUDY/containers/apptainer \
  --evidence-root /home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence \
  --runs-root containers/runs \
  --repository . \
  --output /home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/inventory/image-inventory.json

python3 containers/contracts/image_campaign.py prepare \
  --campaign containers/runs/run-1000-images/config-campaign.json \
  --inventory /home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/inventory/image-inventory.json \
  --catalog containers/contracts/catalog.toml \
  --governance containers/contracts/sites/lunarc.yml \
  --output /home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/campaigns/run-1000-images/execution.json

python3 containers/contracts/image_campaign.py execute \
  --execution /home/bijan/lu2024-12-24/Dardel/STUDY/containers/evidence/campaigns/run-1000-images/execution.json \
  --governance containers/contracts/sites/lunarc.yml \
  --jobs 2
```

Image construction runs on the approved frontend. Real scientific tool runs
remain Slurm jobs with separate pipeline and credential-reference contracts.

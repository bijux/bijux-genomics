# Containers Docs Index

<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: scripts/containers/generate-index.sh -->

Purpose: Authoritative tool/container index for container governance and CI checks.

## Strict TOC
- Entry point: `containers/index.md`
- Policy: `containers/docs/PROMOTION_POLICY.md`
- Lifecycle: `containers/docs/TOOL_LIFECYCLE.md`
- Version authority: `containers/docs/VERSION_AUTHORITY.md`
- Lock lifecycle: `containers/docs/LOCK_LIFECYCLE.md`
- HPC frontend build authority: `containers/docs/FRONTEND_BUILD_AUTHORITY.md`
- Frontend rebuild determinism: `containers/docs/APPTAINER_FRONTEND_REPRODUCIBILITY.md`
- Frontend reproducibility report: `containers/docs/APPTAINER_FRONTEND_REPRODUCIBILITY_REPORT.md`
- Build + style rules: `containers/docs/STYLE.md`
- Smoke: `containers/docs/SMOKE_CONTRACT.md`
- Lock/versioning: `containers/versions/LOCK.md`
- Promotion/demotion: `containers/docs/PROMOTION_POLICY.md`
- Network disclosure: `containers/docs/NETWORK_USAGE.md`
- Security boundary: `containers/docs/SECURITY_BOUNDARY.md`
- Multiarch policy: `containers/docs/MULTIARCH_POLICY.md`
- Licenses: `containers/licenses/`
- SBOM + vulnerability hooks: `scripts/containers/check-sbom-artifacts.sh`, `scripts/containers/check-vuln-hook.sh`
- Exceptions: `containers/docker/NONROOT_EXCEPTIONS.md`, `containers/docker/ENTRYPOINT_EXCEPTIONS.md`, `containers/docs/PLANNED.md`
- Tool ID contract: `containers/docs/TOOL_IDS_CONTRACT.md`

## Authority
- Tool IDs + lifecycle status: `containers/TOOL_IDS.txt` (generated from registry).
- Registry SSoT: `configs/ci/registry/tool_registry*.toml` defines tool existence and lifecycle.
- Container version metadata: `containers/versions/versions.toml` + `containers/versions/lock.json`.
- Non-bijux provenance: `containers/apptainer/non-bijux/NON_BIJUX_SOURCES.md`.
- Ownership map: `containers/OWNERS.toml`.

## Tool Container Coverage
| tool_id | status | apptainer_source | docker_source |
|---|---|---|---|
| `adapterremoval` | `experimental` | `bijux` | `arm64` |
| `addeam` | `experimental` | `bijux` | `arm64` |
| `alientrimmer` | `production` | `bijux` | `arm64` |
| `angsd` | `production` | `bijux` | `arm64` |
| `atropos` | `experimental` | `bijux` | `arm64` |
| `authenticct` | `production` | `bijux` | `arm64` |
| `bamtools` | `production` | `bijux` | `arm64` |
| `bayeshammer` | `experimental` | `bijux` | `arm64` |
| `bbduk` | `production` | `bijux` | `arm64` |
| `bbmerge` | `experimental` | `bijux` | `arm64` |
| `bcftools` | `production` | `non-bijux` | `arm64` |
| `beagle` | `experimental` | `non-bijux` | `arm64` |
| `bedtools` | `production` | `bijux` | `arm64` |
| `bowtie2` | `production` | `bijux` | `arm64` |
| `bracken` | `production` | `bijux` | `arm64` |
| `bwa` | `production` | `bijux` | `arm64` |
| `centrifuge` | `experimental` | `bijux` | `arm64` |
| `contammix` | `production` | `bijux` | `arm64` |
| `cutadapt` | `experimental` | `bijux` | `arm64` |
| `damageprofiler` | `experimental` | `bijux` | `arm64` |
| `eagle` | `experimental` | `non-bijux` | `arm64` |
| `eigensoft` | `experimental` | `non-bijux` | `arm64` |
| `fastp` | `production` | `bijux` | `arm64` |
| `fastq_screen` | `experimental` | `bijux` | `arm64` |
| `fastqc` | `production` | `bijux` | `arm64` |
| `fastqvalidator` | `production` | `bijux` | `arm64` |
| `fastx_clipper` | `production` | `bijux` | `arm64` |
| `flash2` | `experimental` | `bijux` | `arm64` |
| `fqtools` | `experimental` | `bijux` | `arm64` |
| `germline` | `experimental` | `non-bijux` | `arm64` |
| `glimpse` | `planned` | `non-bijux` | `arm64` |
| `ibdhap` | `planned` | `non-bijux` | `arm64` |
| `ibdne` | `planned` | `non-bijux` | `arm64` |
| `ibdseq` | `planned` | `none` | `none` |
| `impute5` | `planned` | `non-bijux` | `arm64` |
| `kaiju` | `experimental` | `bijux` | `arm64` |
| `king` | `production` | `bijux` | `arm64` |
| `kraken2` | `production` | `bijux` | `arm64` |
| `krakenuniq` | `production` | `bijux` | `arm64` |
| `leehom` | `experimental` | `bijux` | `arm64` |
| `lighter` | `experimental` | `bijux` | `arm64` |
| `mapdamage2` | `production` | `bijux` | `arm64` |
| `metaphlan` | `experimental` | `bijux` | `arm64` |
| `minimac4` | `planned` | `non-bijux` | `arm64` |
| `mosdepth` | `production` | `bijux` | `arm64` |
| `multiqc` | `production` | `bijux` | `arm64` |
| `musket` | `experimental` | `bijux` | `arm64` |
| `pear` | `production` | `bijux` | `arm64` |
| `plink` | `experimental` | `bijux` | `arm64` |
| `plink2` | `experimental` | `bijux` | `arm64` |
| `pmdtools` | `production` | `bijux` | `arm64` |
| `prinseq` | `experimental` | `bijux` | `arm64` |
| `pydamage` | `production` | `bijux` | `arm64` |
| `qualimap` | `experimental` | `bijux` | `arm64` |
| `rcorrector` | `production` | `bijux` | `arm64` |
| `rxy` | `production` | `bijux` | `arm64` |
| `samtools` | `production` | `bijux` | `arm64` |
| `schmutzi` | `production` | `bijux` | `arm64` |
| `seqkit` | `production` | `bijux` | `arm64` |
| `seqkit_stats` | `production` | `bijux` | `arm64` |
| `seqtk` | `experimental` | `bijux` | `arm64` |
| `shapeit` | `planned` | `none` | `none` |
| `shapeit5` | `experimental` | `non-bijux` | `arm64` |
| `skewer` | `experimental` | `bijux` | `arm64` |
| `sortmerna` | `production` | `bijux` | `arm64` |
| `spades` | `experimental` | `bijux` | `arm64` |
| `star` | `production` | `bijux` | `arm64` |
| `trim_galore` | `experimental` | `bijux` | `arm64` |
| `trimmomatic` | `experimental` | `bijux` | `arm64` |
| `umi_tools` | `production` | `bijux` | `arm64` |
| `verifybamid2` | `production` | `bijux` | `arm64` |
| `vsearch` | `production` | `bijux` | `arm64` |
| `yleaf` | `experimental` | `bijux` | `arm64` |

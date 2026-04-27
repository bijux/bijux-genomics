# Containers Docs Index

<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: cargo run -p bijux-dna-dev -- containers run generate-index -->

Purpose: Authoritative tool/container index for container governance and CI checks.

## Strict TOC
- Root contract: [containers/README.md](../README.md)
- Entry point: [containers/index.md](../index.md)
- Policy: [containers/docs/PROMOTION_POLICY.md](PROMOTION_POLICY.md)
- Lifecycle: [containers/docs/TOOL_LIFECYCLE.md](TOOL_LIFECYCLE.md)
- Version authority: [containers/docs/VERSION_AUTHORITY.md](VERSION_AUTHORITY.md)
- Lock lifecycle: [containers/docs/LOCK_LIFECYCLE.md](LOCK_LIFECYCLE.md)
- HPC frontend build authority: [containers/docs/FRONTEND_BUILD_AUTHORITY.md](FRONTEND_BUILD_AUTHORITY.md)
- Build + style rules: [containers/docs/STYLE.md](STYLE.md)
- Smoke: [containers/docs/SMOKE_CONTRACT.md](SMOKE_CONTRACT.md)
- Lock/versioning: [containers/versions/LOCK.md](../versions/LOCK.md)
- Network disclosure: [containers/docs/NETWORK_USAGE.md](NETWORK_USAGE.md)
- Security boundary: [containers/docs/SECURITY_BOUNDARY.md](SECURITY_BOUNDARY.md)
- Multiarch policy: [containers/docs/MULTIARCH_POLICY.md](MULTIARCH_POLICY.md)
- GHCR publication: [containers/docs/GHCR_PUBLISH.md](GHCR_PUBLISH.md)
- GHCR packages view: `https://github.com/bijux?tab=packages&repo_name=bijux-genomics`
- Licenses: [containers/licenses/README.md](../licenses/README.md)
- SBOM + vulnerability hooks: `cargo run -p bijux-dna-dev -- containers run check-sbom-artifacts`, `cargo run -p bijux-dna-dev -- containers run check-vuln-hook`
- Exceptions: [containers/docker/NONROOT_EXCEPTIONS.md](../docker/NONROOT_EXCEPTIONS.md), [containers/docker/ENTRYPOINT_EXCEPTIONS.md](../docker/ENTRYPOINT_EXCEPTIONS.md), [containers/docs/PLANNED.md](PLANNED.md)
- Tool ID contract: [containers/docs/TOOL_IDS_CONTRACT.md](TOOL_IDS_CONTRACT.md)

## Authority
- Tool IDs + lifecycle status: [containers/TOOL_IDS.txt](../TOOL_IDS.txt) (generated from registry).
- Registry SSoT: `configs/ci/registry/tool_registry*.toml` defines tool existence and lifecycle.
- Container version metadata: [containers/versions/versions.toml](../versions/versions.toml) + [containers/versions/lock.json](../versions/lock.json).
- GHCR Docker arm64 matrix: `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-publish-matrix -- artifacts/containers/ghcr/docker-arm64-publish-matrix.json`.
- GHCR Apptainer matrix: `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-apptainer-publish-matrix -- artifacts/containers/ghcr/apptainer-publish-matrix.json`.
- Non-bijux provenance: [containers/apptainer/shared/NON_BIJUX_SOURCES.md](../apptainer/shared/NON_BIJUX_SOURCES.md).
- Ownership map: [containers/OWNERS.toml](../OWNERS.toml).

## Tool Container Coverage
| tool_id | status | apptainer_source | docker_source |
|---|---|---|---|
| `adapterremoval` | `production` | `bijux` | `arm64` |
| `addeam` | `experimental` | `bijux` | `arm64` |
| `alientrimmer` | `production` | `bijux` | `arm64` |
| `angsd` | `planned` | `bijux` | `arm64` |
| `atropos` | `production` | `bijux` | `arm64` |
| `authenticct` | `production` | `bijux` | `arm64` |
| `bamtools` | `production` | `bijux` | `arm64` |
| `bamutil` | `experimental` | `bijux` | `arm64` |
| `bayeshammer` | `production` | `bijux` | `arm64` |
| `bbduk` | `production` | `bijux` | `arm64` |
| `bbmerge` | `production` | `bijux` | `arm64` |
| `bcftools` | `production` | `non-bijux` | `arm64` |
| `beagle` | `experimental` | `non-bijux` | `arm64` |
| `bedtools` | `production` | `bijux` | `arm64` |
| `bowtie2` | `production` | `bijux` | `arm64` |
| `bowtie2_build` | `production` | `bijux` | `arm64` |
| `bracken` | `experimental` | `bijux` | `arm64` |
| `bwa` | `production` | `bijux` | `arm64` |
| `centrifuge` | `production` | `bijux` | `arm64` |
| `clumpify` | `production` | `bijux` | `arm64` |
| `contammix` | `production` | `bijux` | `arm64` |
| `cutadapt` | `production` | `bijux` | `arm64` |
| `dada2` | `production` | `bijux` | `arm64` |
| `damageprofiler` | `experimental` | `bijux` | `arm64` |
| `diamond` | `experimental` | `bijux` | `arm64` |
| `dustmasker` | `experimental` | `bijux` | `arm64` |
| `eagle` | `experimental` | `non-bijux` | `arm64` |
| `eigensoft` | `experimental` | `non-bijux` | `arm64` |
| `fastp` | `production` | `bijux` | `arm64` |
| `fastq_scan` | `production` | `bijux` | `arm64` |
| `fastq_screen` | `experimental` | `bijux` | `arm64` |
| `fastqc` | `production` | `bijux` | `arm64` |
| `fastqvalidator` | `production` | `bijux` | `arm64` |
| `fastuniq` | `production` | `bijux` | `arm64` |
| `fastx_clipper` | `production` | `bijux` | `arm64` |
| `flash2` | `production` | `bijux` | `arm64` |
| `fqtools` | `production` | `bijux` | `arm64` |
| `gatk` | `experimental` | `bijux` | `arm64` |
| `germline` | `experimental` | `non-bijux` | `arm64` |
| `glimpse` | `planned` | `non-bijux` | `arm64` |
| `ibdhap` | `planned` | `non-bijux` | `arm64` |
| `ibdne` | `planned` | `non-bijux` | `arm64` |
| `ibdseq` | `planned` | `bijux` | `none` |
| `impute5` | `planned` | `non-bijux` | `arm64` |
| `kaiju` | `production` | `bijux` | `arm64` |
| `king` | `production` | `bijux` | `arm64` |
| `kraken2` | `production` | `bijux` | `arm64` |
| `krakenuniq` | `production` | `bijux` | `arm64` |
| `leehom` | `production` | `bijux` | `arm64` |
| `lighter` | `production` | `bijux` | `arm64` |
| `mapdamage2` | `production` | `bijux` | `arm64` |
| `metaphlan` | `experimental` | `bijux` | `arm64` |
| `minimac4` | `planned` | `non-bijux` | `arm64` |
| `mosdepth` | `production` | `bijux` | `arm64` |
| `multiqc` | `production` | `bijux` | `arm64` |
| `musket` | `production` | `bijux` | `arm64` |
| `ngsbriggs` | `experimental` | `bijux` | `arm64` |
| `pear` | `production` | `bijux` | `arm64` |
| `picard` | `experimental` | `bijux` | `arm64` |
| `plink` | `experimental` | `bijux` | `arm64` |
| `plink2` | `experimental` | `bijux` | `arm64` |
| `pmdtools` | `production` | `bijux` | `arm64` |
| `preseq` | `experimental` | `bijux` | `arm64` |
| `prinseq` | `production` | `bijux` | `arm64` |
| `pydamage` | `production` | `bijux` | `arm64` |
| `qualimap` | `experimental` | `bijux` | `arm64` |
| `rcorrector` | `production` | `bijux` | `arm64` |
| `rxy` | `production` | `bijux` | `arm64` |
| `samtools` | `production` | `bijux` | `arm64` |
| `schmutzi` | `production` | `bijux` | `arm64` |
| `seqfu` | `experimental` | `bijux` | `arm64` |
| `seqkit` | `production` | `bijux` | `arm64` |
| `seqkit_stats` | `production` | `bijux` | `arm64` |
| `seqprep` | `experimental` | `bijux` | `arm64` |
| `seqpurge` | `experimental` | `bijux` | `arm64` |
| `seqtk` | `production` | `bijux` | `arm64` |
| `shapeit` | `planned` | `bijux` | `none` |
| `shapeit5` | `experimental` | `non-bijux` | `arm64` |
| `skewer` | `production` | `bijux` | `arm64` |
| `sortmerna` | `production` | `bijux` | `arm64` |
| `spades` | `experimental` | `bijux` | `arm64` |
| `star` | `production` | `bijux` | `arm64` |
| `trim_galore` | `production` | `bijux` | `arm64` |
| `trimmomatic` | `production` | `bijux` | `arm64` |
| `umi_tools` | `production` | `bijux` | `arm64` |
| `verifybamid2` | `production` | `bijux` | `arm64` |
| `vsearch` | `production` | `bijux` | `arm64` |
| `yleaf` | `experimental` | `bijux` | `arm64` |

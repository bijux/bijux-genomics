# Containers Docs Index

<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: cargo run -p bijux-dna-dev -- containers run generate-index -->

This page routes operators and reviewers to the evidence that supports each
container claim. Recipe presence and registry maturity labels are not proof that
an image builds, starts, processes real data, or produces scientifically useful
output.

## Current evidence

- Image inventory: **current 87, unsupported 2** ([details](../reports/image-inventory.md)).
- Canonical image storage: **passed**, **14 obsolete images quarantined reversibly** ([details](../reports/image-storage.md)).
- Complete isolated rebuild: **passed**, **87 retained images** ([details](../reports/lunarc-image-rebuild.md)).
- Version/help runtime campaign: **passed**, **87 images passed** ([details](../reports/runtime-verification.md)).
- Final run records: **70 tools**, **70 execution passes**; **46** retained tools pass their declared real-data stage and **0** have scientific acceptance ([runs](../reports/run-evidence.md), [qualification](../reports/qualification.md)).
- Build, runtime, real execution, science, benchmark, security, and release are independent evidence dimensions.

## Strict TOC
- Root contract: [containers/README.md](../README.md)
- Entry point: [containers/index.md](../index.md)
- Evidence qualification: [containers/docs/EVIDENCE_QUALIFICATION.md](EVIDENCE_QUALIFICATION.md)
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
- Runtime exceptions: [containers/docker/NONROOT_EXCEPTIONS.md](../docker/NONROOT_EXCEPTIONS.md), [containers/docker/ENTRYPOINT_EXCEPTIONS.md](../docker/ENTRYPOINT_EXCEPTIONS.md)
- Tool ID contract: [containers/docs/TOOL_IDS_CONTRACT.md](TOOL_IDS_CONTRACT.md)

## Authority
- Canonical container identity, support disposition, ownership, runtime support, and evidence requirements: [container catalog](../contracts/catalog.toml).
- Immutable admitted images: [image inventory](../reports/image-inventory.md).
- Canonical image-root reconciliation: [reversible storage evidence](../reports/image-storage.md).
- Rebuild reproducibility: [complete LUNARC image rebuild](../reports/lunarc-image-rebuild.md).
- Support disposition manifest: [containers/TOOL_IDS.txt](../TOOL_IDS.txt); qualification is derived separately from immutable evidence.
- Container version metadata: [versions.toml](../versions/versions.toml) and its generated [lock.json](../versions/lock.json).
- GHCR Docker arm64 matrix: `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-publish-matrix -- artifacts/containers/ghcr/docker-arm64-publish-matrix.json`.
- GHCR Apptainer matrix: `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-apptainer-publish-matrix -- artifacts/containers/ghcr/apptainer-publish-matrix.json`.
- Upstream provenance: [containers/docs/UPSTREAM_SOURCES.md](UPSTREAM_SOURCES.md).

## Container implementation surface

This table identifies retained and unsupported container implementations.
Disposition is not production qualification; use the evidence reports above
for scientific or release decisions.
`apptainer_recipe_owner` describes recipe maintenance, not upstream tool
authorship. Upstream source and license remain catalog-owned facts.

| tool_id | disposition | apptainer_recipe_owner | docker_source |
|---|---|---|---|
| `adapterremoval` | `retained` | `bijux-genomics` | `arm64` |
| `addeam` | `retained` | `bijux-genomics` | `arm64` |
| `alientrimmer` | `retained` | `bijux-genomics` | `arm64` |
| `angsd` | `unsupported` | `none` | `none` |
| `atropos` | `retained` | `bijux-genomics` | `arm64` |
| `authenticct` | `retained` | `bijux-genomics` | `arm64` |
| `bamtools` | `retained` | `bijux-genomics` | `arm64` |
| `bamutil` | `retained` | `bijux-genomics` | `arm64` |
| `bayeshammer` | `retained` | `bijux-genomics` | `arm64` |
| `bbduk` | `retained` | `bijux-genomics` | `arm64` |
| `bbmerge` | `retained` | `bijux-genomics` | `arm64` |
| `bcftools` | `retained` | `bijux-genomics` | `arm64` |
| `beagle` | `retained` | `bijux-genomics` | `arm64` |
| `bedtools` | `retained` | `bijux-genomics` | `arm64` |
| `bijux_dna` | `retained` | `bijux-genomics` | `arm64` |
| `bowtie2` | `retained` | `bijux-genomics` | `arm64` |
| `bowtie2_build` | `retained` | `bijux-genomics` | `arm64` |
| `bracken` | `retained` | `bijux-genomics` | `arm64` |
| `bwa` | `retained` | `bijux-genomics` | `arm64` |
| `centrifuge` | `retained` | `bijux-genomics` | `arm64` |
| `clumpify` | `retained` | `bijux-genomics` | `arm64` |
| `contammix` | `retained` | `bijux-genomics` | `arm64` |
| `cutadapt` | `retained` | `bijux-genomics` | `arm64` |
| `dada2` | `retained` | `bijux-genomics` | `arm64` |
| `damageprofiler` | `retained` | `bijux-genomics` | `arm64` |
| `diamond` | `retained` | `bijux-genomics` | `arm64` |
| `dustmasker` | `retained` | `bijux-genomics` | `arm64` |
| `eagle` | `retained` | `bijux-genomics` | `arm64` |
| `eigensoft` | `retained` | `bijux-genomics` | `arm64` |
| `fastp` | `retained` | `bijux-genomics` | `arm64` |
| `fastq_scan` | `retained` | `bijux-genomics` | `arm64` |
| `fastq_screen` | `retained` | `bijux-genomics` | `arm64` |
| `fastqc` | `retained` | `bijux-genomics` | `arm64` |
| `fastqvalidator` | `retained` | `bijux-genomics` | `arm64` |
| `fastuniq` | `unsupported` | `none` | `none` |
| `fastx_clipper` | `retained` | `bijux-genomics` | `arm64` |
| `flash2` | `retained` | `bijux-genomics` | `arm64` |
| `fqtools` | `retained` | `bijux-genomics` | `arm64` |
| `gatk` | `retained` | `bijux-genomics` | `arm64` |
| `germline` | `retained` | `bijux-genomics` | `arm64` |
| `glimpse` | `retained` | `bijux-genomics` | `arm64` |
| `ibdhap` | `retained` | `bijux-genomics` | `arm64` |
| `ibdne` | `retained` | `bijux-genomics` | `arm64` |
| `ibdseq` | `retained` | `bijux-genomics` | `arm64` |
| `impute5` | `unsupported` | `none` | `none` |
| `kaiju` | `retained` | `bijux-genomics` | `arm64` |
| `king` | `retained` | `bijux-genomics` | `arm64` |
| `kraken2` | `retained` | `bijux-genomics` | `arm64` |
| `krakenuniq` | `retained` | `bijux-genomics` | `arm64` |
| `leehom` | `retained` | `bijux-genomics` | `arm64` |
| `lighter` | `retained` | `bijux-genomics` | `arm64` |
| `mapdamage2` | `retained` | `bijux-genomics` | `arm64` |
| `metaphlan` | `retained` | `bijux-genomics` | `arm64` |
| `minimac4` | `retained` | `bijux-genomics` | `arm64` |
| `mosdepth` | `retained` | `bijux-genomics` | `none` |
| `multiqc` | `retained` | `bijux-genomics` | `arm64` |
| `musket` | `retained` | `bijux-genomics` | `arm64` |
| `ngsbriggs` | `unsupported` | `none` | `none` |
| `pear` | `retained` | `bijux-genomics` | `arm64` |
| `picard` | `retained` | `bijux-genomics` | `arm64` |
| `plink` | `retained` | `bijux-genomics` | `arm64` |
| `plink2` | `retained` | `bijux-genomics` | `arm64` |
| `pmdtools` | `retained` | `bijux-genomics` | `arm64` |
| `preseq` | `retained` | `bijux-genomics` | `arm64` |
| `prinseq` | `retained` | `bijux-genomics` | `arm64` |
| `pydamage` | `retained` | `bijux-genomics` | `arm64` |
| `qualimap` | `retained` | `bijux-genomics` | `arm64` |
| `rcorrector` | `retained` | `bijux-genomics` | `arm64` |
| `rxy` | `retained` | `bijux-genomics` | `arm64` |
| `samtools` | `retained` | `bijux-genomics` | `arm64` |
| `schmutzi` | `retained` | `bijux-genomics` | `arm64` |
| `seqfu` | `retained` | `bijux-genomics` | `arm64` |
| `seqkit` | `retained` | `bijux-genomics` | `arm64` |
| `seqkit_stats` | `retained` | `bijux-genomics` | `arm64` |
| `seqprep` | `retained` | `bijux-genomics` | `arm64` |
| `seqpurge` | `retained` | `bijux-genomics` | `arm64` |
| `seqtk` | `retained` | `bijux-genomics` | `arm64` |
| `shapeit` | `retained` | `bijux-genomics` | `none` |
| `shapeit5` | `retained` | `bijux-genomics` | `none` |
| `skewer` | `retained` | `bijux-genomics` | `arm64` |
| `sortmerna` | `retained` | `bijux-genomics` | `none` |
| `spades` | `retained` | `bijux-genomics` | `arm64` |
| `star` | `retained` | `bijux-genomics` | `arm64` |
| `trim_galore` | `retained` | `bijux-genomics` | `arm64` |
| `trimmomatic` | `retained` | `bijux-genomics` | `arm64` |
| `umi_tools` | `retained` | `bijux-genomics` | `arm64` |
| `verifybamid2` | `retained` | `bijux-genomics` | `arm64` |
| `vsearch` | `retained` | `bijux-genomics` | `arm64` |
| `yleaf` | `retained` | `bijux-genomics` | `arm64` |

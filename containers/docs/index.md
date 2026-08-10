# Containers Docs Index

<!-- GENERATED FILE - DO NOT EDIT -->
<!-- source: cargo run -p bijux-dna-dev -- containers run generate-index -->

This page routes operators and reviewers to the evidence that supports each
container claim. Recipe presence and registry maturity labels are not proof that
an image builds, starts, processes real data, or produces scientifically useful
output.

## Published evidence and current qualification

- Qualification evidence chain: **current**; the current catalog recognizes **83 build**, **85 runtime**, and **83 freshness** passes ([details](../reports/qualification.md)).
- Published image inventory records **current 85, unsupported 4**; their published identities support the current qualification chain ([details](../reports/image-inventory.md)).
- Published canonical storage report: **passed**, **2 obsolete images quarantined reversibly** ([details](../reports/image-storage.md)).
- Published isolated rebuild: **passed**, **87 retained images at that campaign commit** ([details](../reports/lunarc-image-rebuild.md)).
- Published targeted image refresh: **passed**, **1 changed recipe rebuilt and readmitted** ([details](../reports/lunarc-image-refresh.md)).
- Published version/help campaign: **passed**, **85 images passed at that campaign commit** ([details](../reports/runtime-verification.md)).
- Final run records: **71 records**, **71 execution passes**; **47** retained tools pass their declared real-data stage and **0** have scientific acceptance ([runs](../reports/run-evidence.md), [qualification](../reports/qualification.md)).
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
- Canonical container identity, product support, artifact delivery, redistribution permission, ownership, runtime support, and evidence requirements: [container catalog](../contracts/catalog.toml).
- Immutable admitted images: [image inventory](../reports/image-inventory.md).
- Canonical image-root reconciliation: [reversible storage evidence](../reports/image-storage.md).
- Rebuild reproducibility: [complete LUNARC image rebuild](../reports/lunarc-image-rebuild.md).
- Product support and artifact-delivery manifest: [containers/TOOL_IDS.txt](../TOOL_IDS.txt); qualification and external-artifact availability are derived separately from immutable evidence.
- Container version metadata: [versions.toml](../versions/versions.toml) and its generated [lock.json](../versions/lock.json).
- GHCR Docker arm64 matrix: `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-publish-matrix -- artifacts/containers/ghcr/docker-arm64-publish-matrix.json`.
- GHCR Apptainer matrix: `cargo run -q -p bijux-dna-dev -- containers run generate-ghcr-apptainer-publish-matrix -- artifacts/containers/ghcr/apptainer-publish-matrix.json`.
- Upstream provenance: [containers/docs/UPSTREAM_SOURCES.md](UPSTREAM_SOURCES.md).

## Container implementation surface

This table identifies product support independently from artifact delivery and redistribution permission.
These catalog dimensions are not production qualification; use the evidence reports above
for scientific or release decisions.
`apptainer_recipe_owner` describes recipe maintenance, not upstream tool
authorship. Upstream source and license remain catalog-owned facts.

| tool_id | product_support | artifact_delivery | redistribution_permitted | apptainer_recipe_owner | docker_source |
|---|---|---|---|---|---|
| `adapterremoval` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `addeam` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `alientrimmer` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `angsd` | `supported` | `external_artifact` | `false` | `none` | `none` |
| `atropos` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `authenticct` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bamtools` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bamutil` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bayeshammer` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bbduk` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bbmerge` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bcftools` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `beagle` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bedtools` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bijux_dna` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bowtie2` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bowtie2_build` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bracken` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `bwa` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `centrifuge` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `clumpify` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `contammix` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `cutadapt` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `dada2` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `damageprofiler` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `diamond` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `dustmasker` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `eagle` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `eigensoft` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `fastp` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `fastq_scan` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `fastq_screen` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `fastqc` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `fastqvalidator` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `fastuniq` | `supported` | `external_artifact` | `false` | `none` | `none` |
| `fastx_clipper` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `flash2` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `fqtools` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `gatk` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `germline` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `glimpse` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `ibdhap` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `ibdne` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `ibdseq` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `impute5` | `supported` | `external_artifact` | `false` | `none` | `none` |
| `kaiju` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `king` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `kraken2` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `krakenuniq` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `leehom` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `lighter` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `mapdamage2` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `metaphlan` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `minimac4` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `mosdepth` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `none` |
| `multiqc` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `musket` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `ngsbriggs` | `supported` | `external_artifact` | `false` | `none` | `none` |
| `pear` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `picard` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `plink` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `none` |
| `plink2` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `pmdtools` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `preseq` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `prinseq` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `pydamage` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `qualimap` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `rcorrector` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `rxy` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `samtools` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `schmutzi` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `seqfu` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `seqkit` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `seqkit_stats` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `seqprep` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `seqpurge` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `seqtk` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `shapeit` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `none` |
| `shapeit5` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `none` |
| `skewer` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `sortmerna` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `none` |
| `spades` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `star` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `trim_galore` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `trimmomatic` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `umi_tools` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `verifybamid2` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `vsearch` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |
| `yleaf` | `supported` | `redistributable_image` | `true` | `bijux-genomics` | `arm64` |

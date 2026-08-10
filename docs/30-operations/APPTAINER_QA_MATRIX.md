<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- containers run generate-qa-matrix -->

# APPTAINER_QA_MATRIX

## Purpose
Generated matrix for Apptainer-enabled tools, runtime checks, and evidence qualification.

## Scope
The canonical catalog selects redistributable Apptainer images, runtime commands come from the tool registries, and qualification comes from the generated evidence report.

## Non-goals
- Replacing full per-tool smoke manifests.

## Authority Surfaces
- [docs/30-operations/CONTAINERS.md](CONTAINERS.md)
- [docs/30-operations/HPC_FRONTEND_RUNBOOK.md](HPC_FRONTEND_RUNBOOK.md)
- [containers/docs/FRONTEND_BUILD_AUTHORITY.md](../../containers/docs/FRONTEND_BUILD_AUTHORITY.md)
- [containers/docs/SMOKE_CONTRACT.md](../../containers/docs/SMOKE_CONTRACT.md)
- [containers/docs/NETWORK_USAGE.md](../../containers/docs/NETWORK_USAGE.md)
- [containers/reports/qualification.md](../../containers/reports/qualification.md)

## Contracts
- Tool row exists iff the canonical catalog supports a redistributable image and enables its Apptainer distribution.
- `apptainer_def` and smoke command fields are surfaced for QA checks.
- Product support, artifact delivery, and redistribution permission are independent catalog dimensions; qualification is derived independently from immutable evidence.

| Tool ID | Apptainer Def | Smoke Version | Smoke Help | Smoke Minimal | Minimal Exit | Docker Digest | Apptainer Digest | Minimal Rationale | QA Rule | Product support | Artifact delivery | Redistribution permitted | Qualification |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| `adapterremoval` | `containers/apptainer/adapterremoval.def` | `adapterremoval --version` | `adapterremoval --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `addeam` | `containers/apptainer/addeam.def` | `addeam --version` | `addeam --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `alientrimmer` | `containers/apptainer/alientrimmer.def` | `alientrimmer --version` | `alientrimmer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `atropos` | `containers/apptainer/atropos.def` | `atropos --version` | `atropos trim --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `authenticct` | `containers/apptainer/authenticct.def` | `authenticct --version` | `authenticct --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bamtools` | `containers/apptainer/bamtools.def` | `bamtools --version` | `bamtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bamutil` | `containers/apptainer/bamutil.def` | `bamutil --version` | `bamutil --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bayeshammer` | `containers/apptainer/bayeshammer.def` | `bayeshammer --version` | `bayeshammer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bbduk` | `containers/apptainer/bbduk.def` | `bbduk --version` | `bbduk --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bbmerge` | `containers/apptainer/bbmerge.def` | `bbmerge --version` | `bbmerge --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bcftools` | `containers/apptainer/bcftools.def` | `bcftools --version` | `bcftools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `beagle` | `containers/apptainer/beagle.def` | `beagle --version` | `beagle --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bedtools` | `containers/apptainer/bedtools.def` | `bedtools --version` | `bedtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bijux_dna` | `containers/apptainer/bijux_dna.def` | `bijux-dna --version` | `bijux-dna --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bowtie2` | `containers/apptainer/bowtie2.def` | `bowtie2 --version` | `bowtie2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bowtie2_build` | `containers/apptainer/bowtie2_build.def` | `bowtie2-build --version` | `bowtie2-build --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bracken` | `containers/apptainer/bracken.def` | `bracken --version` | `bracken -h` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `bwa` | `containers/apptainer/bwa.def` | `bwa --version` | `bwa --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `centrifuge` | `containers/apptainer/centrifuge.def` | `centrifuge --version` | `centrifuge --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `clumpify` | `containers/apptainer/clumpify.def` | `clumpify --version` | `clumpify --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `contammix` | `containers/apptainer/contammix.def` | `contammix --version` | `contammix --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `cutadapt` | `containers/apptainer/cutadapt.def` | `cutadapt --version` | `cutadapt --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `dada2` | `containers/apptainer/dada2.def` | `dada2 --version` | `dada2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `damageprofiler` | `containers/apptainer/damageprofiler.def` | `damageprofiler --version` | `damageprofiler --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `diamond` | `containers/apptainer/diamond.def` | `diamond version` | `diamond help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `dustmasker` | `containers/apptainer/dustmasker.def` | `dustmasker --version` | `dustmasker --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `eagle` | `containers/apptainer/eagle.def` | `eagle --version` | `eagle --help` | `-` | `0` | `-` | `-` | `No deterministic stage fixture is admitted for this planned tool; help/version probes remain non-qualification checks.` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `eigensoft` | `containers/apptainer/eigensoft.def` | `smartpca --version` | `smartpca --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `fastp` | `containers/apptainer/fastp.def` | `fastp --version` | `fastp --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `fastq_scan` | `containers/apptainer/fastq_scan.def` | `fastq_scan --version` | `fastq_scan --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `fastq_screen` | `containers/apptainer/fastq_screen.def` | `fastq_screen --version` | `fastq_screen --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `fastqc` | `containers/apptainer/fastqc.def` | `fastqc --version` | `fastqc --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `fastqvalidator` | `containers/apptainer/fastqvalidator.def` | `fastqvalidator --version` | `fastqvalidator --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `fastx_clipper` | `containers/apptainer/fastx_clipper.def` | `fastx_clipper --version` | `fastx_clipper --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `flash2` | `containers/apptainer/flash2.def` | `flash2 --version` | `flash2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `fqtools` | `containers/apptainer/fqtools.def` | `fqtools --version` | `fqtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `gatk` | `containers/apptainer/gatk.def` | `gatk --version` | `gatk --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `germline` | `containers/apptainer/germline.def` | `germline --version` | `germline --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `glimpse` | `containers/apptainer/glimpse.def` | `glimpse --version` | `glimpse --help` | `-` | `0` | `-` | `-` | `No deterministic stage fixture is admitted for this planned tool; help/version probes remain non-qualification checks.` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `ibdhap` | `containers/apptainer/ibdhap.def` | `ibdhap --version` | `ibdhap --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `ibdne` | `containers/apptainer/ibdne.def` | `ibdne --version` | `ibdne --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `ibdseq` | `containers/apptainer/ibdseq.def` | `ibdseq --version` | `ibdseq --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `kaiju` | `containers/apptainer/kaiju.def` | `kaiju --version` | `kaiju --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `king` | `containers/apptainer/king.def` | `king --version` | `king --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `kraken2` | `containers/apptainer/kraken2.def` | `kraken2 --version` | `kraken2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `krakenuniq` | `containers/apptainer/krakenuniq.def` | `krakenuniq --version` | `krakenuniq --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `leehom` | `containers/apptainer/leehom.def` | `leehom --version` | `leehom --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `lighter` | `containers/apptainer/lighter.def` | `lighter --version` | `lighter --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `mapdamage2` | `containers/apptainer/mapdamage2.def` | `mapdamage2 --version` | `mapdamage2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `metaphlan` | `containers/apptainer/metaphlan.def` | `metaphlan --version` | `metaphlan --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `minimac4` | `containers/apptainer/minimac4.def` | `minimac4 --version` | `minimac4 --help` | `-` | `0` | `-` | `-` | `No deterministic stage fixture is admitted for this planned tool; help/version probes remain non-qualification checks.` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `mosdepth` | `containers/apptainer/mosdepth.def` | `mosdepth --version` | `mosdepth --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `multiqc` | `containers/apptainer/multiqc.def` | `multiqc --version` | `multiqc --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `musket` | `containers/apptainer/musket.def` | `musket --version` | `musket --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `pear` | `containers/apptainer/pear.def` | `pear --version` | `pear --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `picard` | `containers/apptainer/picard.def` | `picard --version` | `picard --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `plink` | `containers/apptainer/plink.def` | `plink --version` | `plink --help` | `-` | `0` | `-` | `-` | `No deterministic stage fixture is admitted for this planned tool; help/version probes remain non-qualification checks.` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `plink2` | `containers/apptainer/plink2.def` | `plink2 --version` | `plink2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `pmdtools` | `containers/apptainer/pmdtools.def` | `pmdtools --version` | `pmdtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `preseq` | `containers/apptainer/preseq.def` | `preseq` | `preseq` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `prinseq` | `containers/apptainer/prinseq.def` | `prinseq++ --version` | `prinseq++ --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `pydamage` | `containers/apptainer/pydamage.def` | `pydamage --version` | `pydamage --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `qualimap` | `containers/apptainer/qualimap.def` | `qualimap --version` | `qualimap --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `rcorrector` | `containers/apptainer/rcorrector.def` | `rcorrector --version` | `rcorrector --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `rxy` | `containers/apptainer/rxy.def` | `rxy --version` | `rxy --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `samtools` | `containers/apptainer/samtools.def` | `samtools --version` | `samtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `schmutzi` | `containers/apptainer/schmutzi.def` | `schmutzi --version` | `schmutzi --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `seqfu` | `containers/apptainer/seqfu.def` | `seqfu --version` | `seqfu --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `seqkit` | `containers/apptainer/seqkit.def` | `seqkit --version` | `seqkit --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `seqkit_stats` | `containers/apptainer/seqkit_stats.def` | `seqkit_stats --version` | `seqkit_stats --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `seqprep` | `containers/apptainer/seqprep.def` | `seqprep --version` | `seqprep --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `seqpurge` | `containers/apptainer/seqpurge.def` | `seqpurge --version` | `seqpurge --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `seqtk` | `containers/apptainer/seqtk.def` | `seqtk --version` | `seqtk --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `shapeit` | `containers/apptainer/shapeit.def` | `shapeit --version` | `shapeit --help` | `-` | `0` | `-` | `-` | `No deterministic stage fixture is admitted for this planned tool; help/version probes remain non-qualification checks.` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `shapeit5` | `containers/apptainer/shapeit5.def` | `shapeit5 --version` | `shapeit5 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `skewer` | `containers/apptainer/skewer.def` | `skewer --version` | `skewer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `sortmerna` | `containers/apptainer/sortmerna.def` | `sortmerna --version` | `sortmerna -h` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `spades` | `containers/apptainer/spades.def` | `spades.py --version` | `spades.py --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `star` | `containers/apptainer/star.def` | `star --version` | `star --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `trim_galore` | `containers/apptainer/trim_galore.def` | `trim_galore --version` | `trim_galore --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `trimmomatic` | `containers/apptainer/trimmomatic.def` | `trimmomatic --version` | `trimmomatic --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `umi_tools` | `containers/apptainer/umi_tools.def` | `umi_tools --version` | `umi_tools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `verifybamid2` | `containers/apptainer/verifybamid2.def` | `verifybamid2 --version` | `verifybamid2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `vsearch` | `containers/apptainer/vsearch.def` | `vsearch --version` | `vsearch --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |
| `yleaf` | `containers/apptainer/yleaf.def` | `yleaf --version` | `yleaf --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `supported` | `redistributable_image` | `true` | `unverified` |

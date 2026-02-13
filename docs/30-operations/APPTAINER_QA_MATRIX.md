<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: scripts/tooling/generate-apptainer-qa-matrix.sh -->

# APPTAINER_QA_MATRIX

## Purpose
Generated matrix for Apptainer-enabled tools and required QA gates.

## Scope
Derived from tool registries and container metadata fields.

## Non-goals
- Replacing full per-tool smoke manifests.

## Contracts
- Tool row exists iff registry runtimes include `apptainer`.
- `apptainer_def` and smoke command fields are surfaced for QA checks.

| Tool ID | Apptainer Def | Smoke Version | Smoke Help | QA Rule | Status |
|---|---|---|---|---|---|
| `adapterremoval` | `containers/apptainer/bijux/adapterremoval.def` | `adapterremoval --version` | `adapterremoval --help` | `build+smoke required` | `experimental` |
| `addeam` | `containers/apptainer/bijux/addeam.def` | `addeam --version` | `addeam --help` | `build+smoke required` | `experimental` |
| `alientrimmer` | `containers/apptainer/bijux/alientrimmer.def` | `alientrimmer --version` | `alientrimmer --help` | `build+smoke required` | `production` |
| `angsd` | `containers/apptainer/bijux/angsd.def` | `angsd --version` | `angsd --help` | `build+smoke required` | `production` |
| `atropos` | `containers/apptainer/bijux/atropos.def` | `atropos --version` | `atropos --help` | `build+smoke required` | `experimental` |
| `authenticct` | `containers/apptainer/bijux/authenticct.def` | `authenticct --version` | `authenticct --help` | `build+smoke required` | `production` |
| `bamtools` | `containers/apptainer/bijux/bamtools.def` | `bamtools --version` | `bamtools --help` | `build+smoke required` | `production` |
| `bayeshammer` | `containers/apptainer/bijux/bayeshammer.def` | `bayeshammer --version` | `bayeshammer --help` | `build+smoke required` | `experimental` |
| `bbduk` | `containers/apptainer/bijux/bbduk.def` | `bbduk --version` | `bbduk --help` | `build+smoke required` | `production` |
| `bbmerge` | `containers/apptainer/bijux/bbmerge.def` | `bbmerge --version` | `bbmerge --help` | `build+smoke required` | `experimental` |
| `bcftools` | `containers/apptainer/non-bijux/bcftools.def` | `bcftools --version` | `bcftools --help` | `build+smoke required` | `production` |
| `beagle` | `containers/apptainer/non-bijux/beagle.def` | `beagle --version` | `beagle --help` | `build+smoke required` | `planned` |
| `bedtools` | `containers/apptainer/bijux/bedtools.def` | `bedtools --version` | `bedtools --help` | `build+smoke required` | `production` |
| `bowtie2` | `containers/apptainer/bijux/bowtie2.def` | `bowtie2 --version` | `bowtie2 --help` | `build+smoke required` | `production` |
| `bracken` | `containers/apptainer/bijux/bracken.def` | `bracken --version` | `bracken -h` | `build+smoke required` | `production` |
| `bwa` | `containers/apptainer/bijux/bwa.def` | `bwa --version` | `bwa --help` | `build+smoke required` | `production` |
| `centrifuge` | `containers/apptainer/bijux/centrifuge.def` | `centrifuge --version` | `centrifuge --help` | `build+smoke required` | `experimental` |
| `contammix` | `containers/apptainer/bijux/contammix.def` | `contammix --version` | `contammix --help` | `build+smoke required` | `production` |
| `cutadapt` | `containers/apptainer/bijux/cutadapt.def` | `cutadapt --version` | `cutadapt --help` | `build+smoke required` | `experimental` |
| `damageprofiler` | `containers/apptainer/bijux/damageprofiler.def` | `damageprofiler --version` | `damageprofiler --help` | `build+smoke required` | `experimental` |
| `eigensoft` | `containers/apptainer/non-bijux/eigensoft.def` | `smartpca -h` | `smartpca -h` | `build+smoke required` | `planned` |
| `fastp` | `containers/apptainer/bijux/fastp.def` | `fastp --version` | `fastp --help` | `build+smoke required` | `production` |
| `fastq.validate_pre` | `containers/apptainer/bijux/vsearch.def` | `vsearch --version` | `vsearch --help` | `build+smoke required` | `production` |
| `fastq_screen` | `containers/apptainer/bijux/fastq_screen.def` | `fastq_screen --version` | `fastq_screen --help` | `build+smoke required` | `experimental` |
| `fastqc` | `containers/apptainer/bijux/fastqc.def` | `fastqc --version` | `fastqc --help` | `build+smoke required` | `production` |
| `fastqvalidator` | `containers/apptainer/bijux/fastqvalidator.def` | `fastqvalidator --version` | `fastqvalidator --help` | `build+smoke required` | `production` |
| `fastx_clipper` | `containers/apptainer/bijux/fastx_clipper.def` | `fastx_clipper --version` | `fastx_clipper --help` | `build+smoke required` | `production` |
| `flash2` | `containers/apptainer/bijux/flash2.def` | `flash2 --version` | `flash2 --help` | `build+smoke required` | `experimental` |
| `fqtools` | `containers/apptainer/bijux/fqtools.def` | `fqtools --version` | `fqtools --help` | `build+smoke required` | `experimental` |
| `germline` | `containers/apptainer/non-bijux/germline.def` | `germline --version` | `germline --help` | `build+smoke required` | `planned` |
| `ibdhap` | `containers/apptainer/non-bijux/ibdhap.def` | `ibdhap --version` | `ibdhap --help` | `build+smoke required` | `planned` |
| `ibdne` | `containers/apptainer/non-bijux/ibdne.def` | `ibdne --version` | `ibdne --help` | `build+smoke required` | `planned` |
| `ibdseq` | `-` | `ibdseq --version` | `ibdseq --help` | `build+smoke required` | `planned` |
| `kaiju` | `containers/apptainer/bijux/kaiju.def` | `kaiju --version` | `kaiju --help` | `build+smoke required` | `experimental` |
| `king` | `containers/apptainer/bijux/king.def` | `king --version` | `king --help` | `build+smoke required` | `production` |
| `kraken2` | `containers/apptainer/bijux/kraken2.def` | `kraken2 --version` | `kraken2 --help` | `build+smoke required` | `production` |
| `krakenuniq` | `containers/apptainer/bijux/krakenuniq.def` | `krakenuniq --version` | `krakenuniq --help` | `build+smoke required` | `production` |
| `leehom` | `containers/apptainer/bijux/leehom.def` | `leehom --version` | `leehom --help` | `build+smoke required` | `experimental` |
| `lighter` | `containers/apptainer/bijux/lighter.def` | `lighter --version` | `lighter --help` | `build+smoke required` | `experimental` |
| `mapdamage2` | `containers/apptainer/bijux/mapdamage2.def` | `mapdamage2 --version` | `mapdamage2 --help` | `build+smoke required` | `production` |
| `metaphlan` | `containers/apptainer/bijux/metaphlan.def` | `metaphlan --version` | `metaphlan --help` | `build+smoke required` | `experimental` |
| `mosdepth` | `containers/apptainer/bijux/mosdepth.def` | `mosdepth --version` | `mosdepth --help` | `build+smoke required` | `production` |
| `multiqc` | `containers/apptainer/bijux/multiqc.def` | `multiqc --version` | `multiqc --help` | `build+smoke required` | `production` |
| `musket` | `containers/apptainer/bijux/musket.def` | `musket --version` | `musket --help` | `build+smoke required` | `experimental` |
| `pear` | `containers/apptainer/bijux/pear.def` | `pear --version` | `pear --help` | `build+smoke required` | `production` |
| `plink` | `containers/apptainer/bijux/plink.def` | `plink --version` | `plink --help` | `build+smoke required` | `planned` |
| `plink2` | `containers/apptainer/bijux/plink2.def` | `plink2 --version` | `plink2 --help` | `build+smoke required` | `planned` |
| `pmdtools` | `containers/apptainer/bijux/pmdtools.def` | `pmdtools --version` | `pmdtools --help` | `build+smoke required` | `production` |
| `prinseq` | `containers/apptainer/bijux/prinseq.def` | `prinseq++ --version` | `prinseq++ --help` | `build+smoke required` | `experimental` |
| `pydamage` | `containers/apptainer/bijux/pydamage.def` | `pydamage --version` | `pydamage --help` | `build+smoke required` | `production` |
| `qualimap` | `containers/apptainer/bijux/qualimap.def` | `qualimap --version` | `qualimap --help` | `build+smoke required` | `experimental` |
| `rcorrector` | `containers/apptainer/bijux/rcorrector.def` | `rcorrector --version` | `rcorrector --help` | `build+smoke required` | `production` |
| `rxy` | `containers/apptainer/bijux/rxy.def` | `rxy --version` | `rxy --help` | `build+smoke required` | `production` |
| `samtools` | `containers/apptainer/bijux/samtools.def` | `samtools --version` | `samtools --help` | `build+smoke required` | `production` |
| `schmutzi` | `containers/apptainer/bijux/schmutzi.def` | `schmutzi --version` | `schmutzi --help` | `build+smoke required` | `production` |
| `seqkit` | `containers/apptainer/bijux/seqkit.def` | `seqkit --version` | `seqkit --help` | `build+smoke required` | `production` |
| `seqkit_stats` | `containers/apptainer/bijux/seqkit_stats.def` | `seqkit_stats --version` | `seqkit_stats --help` | `build+smoke required` | `production` |
| `seqtk` | `containers/apptainer/bijux/seqtk.def` | `seqtk --version` | `seqtk --help` | `build+smoke required` | `experimental` |
| `shapeit` | `-` | `shapeit --version` | `shapeit --help` | `build+smoke required` | `planned` |
| `skewer` | `containers/apptainer/bijux/skewer.def` | `skewer --version` | `skewer --help` | `build+smoke required` | `experimental` |
| `sortmerna` | `containers/apptainer/bijux/sortmerna.def` | `sortmerna --version` | `sortmerna --help` | `build+smoke required` | `production` |
| `spades` | `containers/apptainer/bijux/spades.def` | `spades --version` | `spades --help` | `build+smoke required` | `experimental` |
| `star` | `containers/apptainer/bijux/star.def` | `star --version` | `star --help` | `build+smoke required` | `production` |
| `trim_galore` | `containers/apptainer/bijux/trim_galore.def` | `trim_galore --version` | `trim_galore --help` | `build+smoke required` | `experimental` |
| `trimmomatic` | `containers/apptainer/bijux/trimmomatic.def` | `trimmomatic --version` | `trimmomatic --help` | `build+smoke required` | `experimental` |
| `umi_tools` | `containers/apptainer/bijux/umi_tools.def` | `umi_tools --version` | `umi_tools --help` | `build+smoke required` | `production` |
| `verifybamid2` | `containers/apptainer/bijux/verifybamid2.def` | `verifybamid2 --version` | `verifybamid2 --help` | `build+smoke required` | `production` |
| `yleaf` | `containers/apptainer/bijux/yleaf.def` | `yleaf --version` | `yleaf --help` | `build+smoke required` | `experimental` |

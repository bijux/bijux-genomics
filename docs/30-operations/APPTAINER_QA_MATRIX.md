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

| Tool ID | Apptainer Def | Smoke Command | QA Rule | Status |
|---|---|---|---|---|
| `adapterremoval` | `containers/apptainer/bijux/adapterremoval.def` | `adapterremoval --help` | `build+smoke required` | `supported` |
| `addeam` | `containers/apptainer/bijux/addeam.def` | `addeam --help` | `build+smoke required` | `supported` |
| `alientrimmer` | `containers/apptainer/bijux/alientrimmer.def` | `alientrimmer --help` | `build+smoke required` | `supported` |
| `angsd` | `containers/apptainer/bijux/angsd.def` | `angsd --help` | `build+smoke required` | `supported` |
| `atropos` | `containers/apptainer/bijux/atropos.def` | `atropos --help` | `build+smoke required` | `supported` |
| `authenticct` | `containers/apptainer/bijux/authenticct.def` | `authenticct --help` | `build+smoke required` | `supported` |
| `bamtools` | `containers/apptainer/bijux/bamtools.def` | `bamtools --help` | `build+smoke required` | `supported` |
| `bayeshammer` | `containers/apptainer/bijux/bayeshammer.def` | `bayeshammer --help` | `build+smoke required` | `supported` |
| `bbduk` | `containers/apptainer/bijux/bbduk.def` | `bbduk --help` | `build+smoke required` | `supported` |
| `bbmerge` | `containers/apptainer/bijux/bbmerge.def` | `bbmerge --help` | `build+smoke required` | `supported` |
| `bcftools` | `containers/apptainer/non-bijux/bcftools.def` | `bcftools --help` | `build+smoke required` | `supported` |
| `bedtools` | `containers/apptainer/bijux/bedtools.def` | `bedtools --help` | `build+smoke required` | `supported` |
| `bowtie2` | `containers/apptainer/bijux/bowtie2.def` | `bowtie2 --help` | `build+smoke required` | `supported` |
| `bracken` | `containers/apptainer/bijux/bracken.def` | `bracken -h` | `build+smoke required` | `supported` |
| `bwa` | `containers/apptainer/bijux/bwa.def` | `bwa --help` | `build+smoke required` | `supported` |
| `centrifuge` | `containers/apptainer/bijux/centrifuge.def` | `centrifuge --help` | `build+smoke required` | `supported` |
| `contammix` | `containers/apptainer/bijux/contammix.def` | `contammix --help` | `build+smoke required` | `supported` |
| `cutadapt` | `containers/apptainer/bijux/cutadapt.def` | `cutadapt --help` | `build+smoke required` | `supported` |
| `damageprofiler` | `containers/apptainer/bijux/damageprofiler.def` | `damageprofiler --help` | `build+smoke required` | `supported` |
| `fastp` | `containers/apptainer/bijux/fastp.def` | `fastp --help` | `build+smoke required` | `supported` |
| `fastq.validate_pre` | `containers/apptainer/bijux/vsearch.def` | `vsearch --help` | `build+smoke required` | `supported` |
| `fastq_screen` | `containers/apptainer/bijux/fastq_screen.def` | `fastq_screen --help` | `build+smoke required` | `supported` |
| `fastqc` | `containers/apptainer/bijux/fastqc.def` | `fastqc --help` | `build+smoke required` | `supported` |
| `fastqvalidator` | `containers/apptainer/bijux/fastqvalidator.def` | `fastqvalidator --help` | `build+smoke required` | `supported` |
| `fastx_clipper` | `containers/apptainer/bijux/fastx_clipper.def` | `fastx_clipper --help` | `build+smoke required` | `supported` |
| `flash2` | `containers/apptainer/bijux/flash2.def` | `flash2 --help` | `build+smoke required` | `supported` |
| `fqtools` | `containers/apptainer/bijux/fqtools.def` | `fqtools --help` | `build+smoke required` | `supported` |
| `kaiju` | `containers/apptainer/bijux/kaiju.def` | `kaiju --help` | `build+smoke required` | `supported` |
| `king` | `containers/apptainer/bijux/king.def` | `king --help` | `build+smoke required` | `supported` |
| `kraken2` | `containers/apptainer/bijux/kraken2.def` | `kraken2 --help` | `build+smoke required` | `supported` |
| `krakenuniq` | `containers/apptainer/bijux/krakenuniq.def` | `krakenuniq --help` | `build+smoke required` | `supported` |
| `leehom` | `containers/apptainer/bijux/leehom.def` | `leehom --help` | `build+smoke required` | `supported` |
| `lighter` | `containers/apptainer/bijux/lighter.def` | `lighter --help` | `build+smoke required` | `supported` |
| `mapdamage2` | `containers/apptainer/bijux/mapdamage2.def` | `mapdamage2 --help` | `build+smoke required` | `supported` |
| `metaphlan` | `containers/apptainer/bijux/metaphlan.def` | `metaphlan --help` | `build+smoke required` | `supported` |
| `mosdepth` | `containers/apptainer/bijux/mosdepth.def` | `mosdepth --help` | `build+smoke required` | `supported` |
| `multiqc` | `containers/apptainer/bijux/multiqc.def` | `multiqc --help` | `build+smoke required` | `supported` |
| `musket` | `containers/apptainer/bijux/musket.def` | `musket --help` | `build+smoke required` | `supported` |
| `pear` | `containers/apptainer/bijux/pear.def` | `pear --help` | `build+smoke required` | `supported` |
| `pmdtools` | `containers/apptainer/bijux/pmdtools.def` | `pmdtools --help` | `build+smoke required` | `supported` |
| `prinseq` | `containers/apptainer/bijux/prinseq.def` | `prinseq++ --help` | `build+smoke required` | `supported` |
| `pydamage` | `containers/apptainer/bijux/pydamage.def` | `pydamage --help` | `build+smoke required` | `supported` |
| `qualimap` | `containers/apptainer/bijux/qualimap.def` | `qualimap --help` | `build+smoke required` | `supported` |
| `rcorrector` | `containers/apptainer/bijux/rcorrector.def` | `rcorrector --help` | `build+smoke required` | `supported` |
| `rxy` | `containers/apptainer/bijux/rxy.def` | `rxy --help` | `build+smoke required` | `supported` |
| `samtools` | `containers/apptainer/bijux/samtools.def` | `samtools --help` | `build+smoke required` | `supported` |
| `schmutzi` | `containers/apptainer/bijux/schmutzi.def` | `schmutzi --help` | `build+smoke required` | `supported` |
| `seqkit` | `containers/apptainer/bijux/seqkit.def` | `seqkit --help` | `build+smoke required` | `supported` |
| `seqkit_stats` | `containers/apptainer/bijux/seqkit_stats.def` | `seqkit_stats --help` | `build+smoke required` | `supported` |
| `seqtk` | `containers/apptainer/bijux/seqtk.def` | `seqtk --help` | `build+smoke required` | `supported` |
| `skewer` | `containers/apptainer/bijux/skewer.def` | `skewer --help` | `build+smoke required` | `supported` |
| `sortmerna` | `containers/apptainer/bijux/sortmerna.def` | `sortmerna --help` | `build+smoke required` | `supported` |
| `spades` | `containers/apptainer/bijux/spades.def` | `spades --help` | `build+smoke required` | `supported` |
| `star` | `containers/apptainer/bijux/star.def` | `star --help` | `build+smoke required` | `supported` |
| `trim_galore` | `containers/apptainer/bijux/trim_galore.def` | `trim_galore --help` | `build+smoke required` | `supported` |
| `trimmomatic` | `containers/apptainer/bijux/trimmomatic.def` | `trimmomatic --help` | `build+smoke required` | `supported` |
| `umi_tools` | `containers/apptainer/bijux/umi_tools.def` | `umi_tools --help` | `build+smoke required` | `supported` |
| `verifybamid2` | `containers/apptainer/bijux/verifybamid2.def` | `verifybamid2 --help` | `build+smoke required` | `supported` |
| `yleaf` | `containers/apptainer/bijux/yleaf.def` | `yleaf --help` | `build+smoke required` | `supported` |

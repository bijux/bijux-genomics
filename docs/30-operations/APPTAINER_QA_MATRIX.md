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

| Tool ID | Apptainer Def | Smoke Version | Smoke Help | Smoke Minimal | Minimal Exit | Minimal Rationale | QA Rule | Status |
|---|---|---|---|---|---|---|---|---|
| `adapterremoval` | `containers/apptainer/bijux/adapterremoval.def` | `adapterremoval --version` | `adapterremoval --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `addeam` | `containers/apptainer/bijux/addeam.def` | `addeam --version` | `addeam --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `alientrimmer` | `containers/apptainer/bijux/alientrimmer.def` | `alientrimmer --version` | `alientrimmer --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `angsd` | `containers/apptainer/non-bijux/bcftools.def` | `angsd 2>&1 | head -n 1` | `angsd -h` | `-` | `0` | `minimal command contract` | `build+smoke required` | `planned` |
| `atropos` | `containers/apptainer/bijux/atropos.def` | `atropos --version` | `atropos --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `authenticct` | `containers/apptainer/bijux/authenticct.def` | `authenticct --version` | `authenticct --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `bamtools` | `containers/apptainer/bijux/bamtools.def` | `bamtools --version` | `bamtools --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `bayeshammer` | `containers/apptainer/bijux/bayeshammer.def` | `bayeshammer --version` | `bayeshammer --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `bbduk` | `containers/apptainer/bijux/bbduk.def` | `bbduk --version` | `bbduk --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `bbmerge` | `containers/apptainer/bijux/bbmerge.def` | `bbmerge --version` | `bbmerge --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `bcftools` | `containers/apptainer/non-bijux/bcftools.def` | `bcftools --version` | `bcftools --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `beagle` | `containers/apptainer/non-bijux/beagle.def` | `beagle --version` | `beagle --help` | `beagle --help` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `bedtools` | `containers/apptainer/bijux/bedtools.def` | `bedtools --version` | `bedtools --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `bowtie2` | `containers/apptainer/bijux/bowtie2.def` | `bowtie2 --version` | `bowtie2 --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `bracken` | `containers/apptainer/bijux/bracken.def` | `bracken --version` | `bracken -h` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `bwa` | `containers/apptainer/bijux/bwa.def` | `bwa --version` | `bwa --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `centrifuge` | `containers/apptainer/bijux/centrifuge.def` | `centrifuge --version` | `centrifuge --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `contammix` | `containers/apptainer/bijux/contammix.def` | `contammix --version` | `contammix --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `cutadapt` | `containers/apptainer/bijux/cutadapt.def` | `cutadapt --version` | `cutadapt --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `damageprofiler` | `containers/apptainer/bijux/damageprofiler.def` | `damageprofiler --version` | `damageprofiler --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `eagle` | `containers/apptainer/non-bijux/eagle.def` | `eagle --version` | `eagle --help` | `eagle --help` | `0` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `experimental` |
| `eigensoft` | `containers/apptainer/non-bijux/eigensoft.def` | `smartpca -h` | `smartpca -h` | `smartpca -h` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `fastp` | `containers/apptainer/bijux/fastp.def` | `fastp --version` | `fastp --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `fastq.validate_pre` | `containers/apptainer/bijux/vsearch.def` | `vsearch --version` | `vsearch --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `fastq_screen` | `containers/apptainer/bijux/fastq_screen.def` | `fastq_screen --version` | `fastq_screen --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `fastqc` | `containers/apptainer/bijux/fastqc.def` | `fastqc --version` | `fastqc --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `fastqvalidator` | `containers/apptainer/bijux/fastqvalidator.def` | `fastqvalidator --version` | `fastqvalidator --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `fastx_clipper` | `containers/apptainer/bijux/fastx_clipper.def` | `fastx_clipper --version` | `fastx_clipper --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `flash2` | `containers/apptainer/bijux/flash2.def` | `flash2 --version` | `flash2 --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `fqtools` | `containers/apptainer/bijux/fqtools.def` | `fqtools --version` | `fqtools --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `germline` | `containers/apptainer/non-bijux/germline.def` | `germline --version` | `germline --help` | `germline --help` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `glimpse` | `containers/apptainer/non-bijux/glimpse.def` | `glimpse --version` | `glimpse --help` | `glimpse --help` | `0` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `planned` |
| `ibdhap` | `containers/apptainer/non-bijux/ibdhap.def` | `ibdhap --version` | `ibdhap --help` | `ibdhap --help` | `0` | `minimal command contract` | `build+smoke required` | `planned` |
| `ibdne` | `containers/apptainer/non-bijux/ibdne.def` | `ibdne --version` | `ibdne --help` | `ibdne --help` | `0` | `minimal command contract` | `build+smoke required` | `planned` |
| `ibdseq` | `-` | `ibdseq --version` | `ibdseq --help` | `ibdseq --help` | `0` | `minimal command contract` | `build+smoke required` | `planned` |
| `impute5` | `containers/apptainer/non-bijux/impute5.def` | `impute5 --version` | `impute5 --help` | `impute5 --help` | `0` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `planned` |
| `kaiju` | `containers/apptainer/bijux/kaiju.def` | `kaiju --version` | `kaiju --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `king` | `containers/apptainer/bijux/king.def` | `king --version` | `king --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `kraken2` | `containers/apptainer/bijux/kraken2.def` | `kraken2 --version` | `kraken2 --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `krakenuniq` | `containers/apptainer/bijux/krakenuniq.def` | `krakenuniq --version` | `krakenuniq --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `leehom` | `containers/apptainer/bijux/leehom.def` | `leehom --version` | `leehom --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `lighter` | `containers/apptainer/bijux/lighter.def` | `lighter --version` | `lighter --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `mapdamage2` | `containers/apptainer/bijux/mapdamage2.def` | `mapdamage2 --version` | `mapdamage2 --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `metaphlan` | `containers/apptainer/bijux/metaphlan.def` | `metaphlan --version` | `metaphlan --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `minimac4` | `containers/apptainer/non-bijux/minimac4.def` | `minimac4 --version` | `minimac4 --help` | `minimac4 --help` | `0` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `planned` |
| `mosdepth` | `containers/apptainer/bijux/mosdepth.def` | `mosdepth --version` | `mosdepth --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `multiqc` | `containers/apptainer/bijux/multiqc.def` | `multiqc --version` | `multiqc --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `musket` | `containers/apptainer/bijux/musket.def` | `musket --version` | `musket --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `pear` | `containers/apptainer/bijux/pear.def` | `pear --version` | `pear --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `plink` | `containers/apptainer/bijux/plink.def` | `plink --version` | `plink --help` | `tmp=$(mktemp -d); printf 'FAM1 S1 0 0 1 1\\n' > \"$tmp/tiny.ped\"; printf '1 rs1 0 1000 A G\\n' > \"$tmp/tiny.map\"; plink --file \"$tmp/tiny\" --freq --allow-no-sex --out \"$tmp/out\" >/dev/null 2>&1; test -s \"$tmp/out.frq\` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `plink2` | `containers/apptainer/bijux/plink2.def` | `plink2 --version` | `plink2 --help` | `tmp=$(mktemp -d); printf 'FAM1 S1 0 0 1 1\\n' > \"$tmp/tiny.ped\"; printf '1 rs1 0 1000 A G\\n' > \"$tmp/tiny.map\"; plink2 --pedmap \"$tmp/tiny\" --freq --allow-no-sex --out \"$tmp/out\" >/dev/null 2>&1; test -s \"$tmp/out.afreq\` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `pmdtools` | `containers/apptainer/bijux/pmdtools.def` | `pmdtools --version` | `pmdtools --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `prinseq` | `containers/apptainer/bijux/prinseq.def` | `prinseq++ --version` | `prinseq++ --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `pydamage` | `containers/apptainer/bijux/pydamage.def` | `pydamage --version` | `pydamage --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `qualimap` | `containers/apptainer/bijux/qualimap.def` | `qualimap --version` | `qualimap --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `rcorrector` | `containers/apptainer/bijux/rcorrector.def` | `rcorrector --version` | `rcorrector --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `rxy` | `containers/apptainer/bijux/rxy.def` | `rxy --version` | `rxy --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `samtools` | `containers/apptainer/bijux/samtools.def` | `samtools --version` | `samtools --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `schmutzi` | `containers/apptainer/bijux/schmutzi.def` | `schmutzi --version` | `schmutzi --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `seqkit` | `containers/apptainer/bijux/seqkit.def` | `seqkit --version` | `seqkit --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `seqkit_stats` | `containers/apptainer/bijux/seqkit_stats.def` | `seqkit_stats --version` | `seqkit_stats --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `seqtk` | `containers/apptainer/bijux/seqtk.def` | `seqtk --version` | `seqtk --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `shapeit` | `-` | `shapeit --version` | `shapeit --help` | `shapeit --help` | `0` | `minimal command contract` | `build+smoke required` | `planned` |
| `shapeit5` | `containers/apptainer/non-bijux/shapeit5.def` | `shapeit5 --version` | `shapeit5 --help` | `shapeit5 --help` | `0` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `experimental` |
| `skewer` | `containers/apptainer/bijux/skewer.def` | `skewer --version` | `skewer --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `sortmerna` | `containers/apptainer/bijux/sortmerna.def` | `sortmerna --version` | `sortmerna --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `spades` | `containers/apptainer/bijux/spades.def` | `spades --version` | `spades --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `star` | `containers/apptainer/bijux/star.def` | `star --version` | `star --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `trim_galore` | `containers/apptainer/bijux/trim_galore.def` | `trim_galore --version` | `trim_galore --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `trimmomatic` | `containers/apptainer/bijux/trimmomatic.def` | `trimmomatic --version` | `trimmomatic --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |
| `umi_tools` | `containers/apptainer/bijux/umi_tools.def` | `umi_tools --version` | `umi_tools --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `verifybamid2` | `containers/apptainer/bijux/verifybamid2.def` | `verifybamid2 --version` | `verifybamid2 --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `production` |
| `yleaf` | `containers/apptainer/bijux/yleaf.def` | `yleaf --version` | `yleaf --help` | `-` | `0` | `minimal command contract` | `build+smoke required` | `experimental` |

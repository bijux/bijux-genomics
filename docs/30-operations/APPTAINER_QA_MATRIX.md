<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- containers run generate-qa-matrix -->

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

| Tool ID | Apptainer Def | Smoke Version | Smoke Help | Smoke Minimal | Minimal Exit | Docker Digest | Apptainer Digest | Minimal Rationale | QA Rule | Status |
|---|---|---|---|---|---|---|---|---|---|---|
| `adapterremoval` | `containers/apptainer/lunarc/adapterremoval.def` | `adapterremoval --version` | `adapterremoval --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `addeam` | `containers/apptainer/lunarc/addeam.def` | `addeam --version` | `addeam --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `alientrimmer` | `containers/apptainer/lunarc/alientrimmer.def` | `alientrimmer --version` | `alientrimmer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `angsd` | `containers/apptainer/lunarc/angsd.def` | `angsd 2>&1 | head -n 1` | `angsd -h` | `angsd -h` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `planned` |
| `atropos` | `containers/apptainer/lunarc/atropos.def` | `atropos --version` | `atropos --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `authenticct` | `containers/apptainer/lunarc/authenticct.def` | `authenticct --version` | `authenticct --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bamtools` | `containers/apptainer/lunarc/bamtools.def` | `bamtools --version` | `bamtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bayeshammer` | `containers/apptainer/lunarc/bayeshammer.def` | `bayeshammer --version` | `bayeshammer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `bbduk` | `containers/apptainer/lunarc/bbduk.def` | `bbduk --version` | `bbduk --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bbmerge` | `containers/apptainer/lunarc/bbmerge.def` | `bbmerge --version` | `bbmerge --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `bcftools` | `containers/apptainer/lunarc/bcftools.def` | `bcftools --version` | `bcftools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `beagle` | `containers/apptainer/lunarc/beagle.def` | `beagle --version` | `beagle --help` | `beagle --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `bedtools` | `containers/apptainer/lunarc/bedtools.def` | `bedtools --version` | `bedtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bowtie2` | `containers/apptainer/lunarc/bowtie2.def` | `bowtie2 --version` | `bowtie2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bracken` | `containers/apptainer/lunarc/bracken.def` | `bracken --version` | `bracken -h` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bwa` | `containers/apptainer/lunarc/bwa.def` | `bwa --version` | `bwa --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `centrifuge` | `containers/apptainer/lunarc/centrifuge.def` | `centrifuge --version` | `centrifuge --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `contammix` | `containers/apptainer/lunarc/contammix.def` | `contammix --version` | `contammix --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `cutadapt` | `containers/apptainer/lunarc/cutadapt.def` | `cutadapt --version` | `cutadapt --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `damageprofiler` | `containers/apptainer/lunarc/damageprofiler.def` | `damageprofiler --version` | `damageprofiler --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `eagle` | `containers/apptainer/lunarc/eagle.def` | `eagle --version` | `eagle --help` | `eagle --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `experimental` |
| `eigensoft` | `containers/apptainer/lunarc/eigensoft.def` | `smartpca --version` | `smartpca --help` | `smartpca --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `fastp` | `containers/apptainer/lunarc/fastp.def` | `fastp --version` | `fastp --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `fastq_screen` | `containers/apptainer/lunarc/fastq_screen.def` | `fastq_screen --version` | `fastq_screen --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `fastqc` | `containers/apptainer/lunarc/fastqc.def` | `fastqc --version` | `fastqc --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `fastqvalidator` | `containers/apptainer/lunarc/fastqvalidator.def` | `fastqvalidator --version` | `fastqvalidator --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `fastx_clipper` | `containers/apptainer/lunarc/fastx_clipper.def` | `fastx_clipper --version` | `fastx_clipper --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `flash2` | `containers/apptainer/lunarc/flash2.def` | `flash2 --version` | `flash2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `fqtools` | `containers/apptainer/lunarc/fqtools.def` | `fqtools --version` | `fqtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `germline` | `containers/apptainer/lunarc/germline.def` | `germline --version` | `germline --help` | `germline --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `glimpse` | `containers/apptainer/lunarc/glimpse.def` | `glimpse --version` | `glimpse --help` | `glimpse --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `planned` |
| `ibdhap` | `containers/apptainer/lunarc/ibdhap.def` | `ibdhap --version` | `ibdhap --help` | `ibdhap --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `planned` |
| `ibdne` | `containers/apptainer/lunarc/ibdne.def` | `ibdne --version` | `ibdne --help` | `ibdne --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `planned` |
| `ibdseq` | `-` | `ibdseq --version` | `ibdseq --help` | `ibdseq --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `planned` |
| `impute5` | `containers/apptainer/lunarc/impute5.def` | `impute5 --version` | `impute5 --help` | `impute5 --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `planned` |
| `kaiju` | `containers/apptainer/lunarc/kaiju.def` | `kaiju --version` | `kaiju --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `king` | `containers/apptainer/lunarc/king.def` | `king --version` | `king --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `kraken2` | `containers/apptainer/lunarc/kraken2.def` | `kraken2 --version` | `kraken2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `krakenuniq` | `containers/apptainer/lunarc/krakenuniq.def` | `krakenuniq --version` | `krakenuniq --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `leehom` | `containers/apptainer/lunarc/leehom.def` | `leehom --version` | `leehom --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `lighter` | `containers/apptainer/lunarc/lighter.def` | `lighter --version` | `lighter --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `mapdamage2` | `containers/apptainer/lunarc/mapdamage2.def` | `mapdamage2 --version` | `mapdamage2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `metaphlan` | `containers/apptainer/lunarc/metaphlan.def` | `metaphlan --version` | `metaphlan --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `minimac4` | `containers/apptainer/lunarc/minimac4.def` | `minimac4 --version` | `minimac4 --help` | `minimac4 --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `planned` |
| `mosdepth` | `containers/apptainer/lunarc/mosdepth.def` | `mosdepth --version` | `mosdepth --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `multiqc` | `containers/apptainer/lunarc/multiqc.def` | `multiqc --version` | `multiqc --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `musket` | `containers/apptainer/lunarc/musket.def` | `musket --version` | `musket --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `pear` | `containers/apptainer/lunarc/pear.def` | `pear --version` | `pear --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `plink` | `containers/apptainer/lunarc/plink.def` | `plink --version` | `plink --help` | `tmp=$(mktemp -d); printf 'FAM1 S1 0 0 1 1\\n' > \"$tmp/tiny.ped\"; printf '1 rs1 0 1000 A G\\n' > \"$tmp/tiny.map\"; plink --file \"$tmp/tiny\" --freq --allow-no-sex --out \"$tmp/out\" >/dev/null 2>&1; test -s \"$tmp/out.frq\"` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `plink2` | `containers/apptainer/lunarc/plink2.def` | `plink2 --version` | `plink2 --help` | `tmp=$(mktemp -d); printf 'FAM1 S1 0 0 1 1\\n' > \"$tmp/tiny.ped\"; printf '1 rs1 0 1000 A G\\n' > \"$tmp/tiny.map\"; plink2 --pedmap \"$tmp/tiny\" --freq --allow-no-sex --out \"$tmp/out\" >/dev/null 2>&1; test -s \"$tmp/out.afreq\"` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `pmdtools` | `containers/apptainer/lunarc/pmdtools.def` | `pmdtools --version` | `pmdtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `prinseq` | `containers/apptainer/lunarc/prinseq.def` | `prinseq++ --version` | `prinseq++ --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `pydamage` | `containers/apptainer/lunarc/pydamage.def` | `pydamage --version` | `pydamage --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `qualimap` | `containers/apptainer/lunarc/qualimap.def` | `qualimap --version` | `qualimap --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `rcorrector` | `containers/apptainer/lunarc/rcorrector.def` | `rcorrector --version` | `rcorrector --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `rxy` | `containers/apptainer/lunarc/rxy.def` | `rxy --version` | `rxy --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `samtools` | `containers/apptainer/lunarc/samtools.def` | `samtools --version` | `samtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `schmutzi` | `containers/apptainer/lunarc/schmutzi.def` | `schmutzi --version` | `schmutzi --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `seqkit` | `containers/apptainer/lunarc/seqkit.def` | `seqkit --version` | `seqkit --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `seqkit_stats` | `containers/apptainer/lunarc/seqkit_stats.def` | `seqkit_stats --version` | `seqkit_stats --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `seqtk` | `containers/apptainer/lunarc/seqtk.def` | `seqtk --version` | `seqtk --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `shapeit` | `-` | `shapeit --version` | `shapeit --help` | `shapeit --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `planned` |
| `shapeit5` | `containers/apptainer/lunarc/shapeit5.def` | `shapeit5 --version` | `shapeit5 --help` | `shapeit5 --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `experimental` |
| `skewer` | `containers/apptainer/lunarc/skewer.def` | `skewer --version` | `skewer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `sortmerna` | `containers/apptainer/lunarc/sortmerna.def` | `sortmerna --version` | `sortmerna --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `spades` | `containers/apptainer/lunarc/spades.def` | `spades --version` | `spades --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `star` | `containers/apptainer/lunarc/star.def` | `star --version` | `star --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `trim_galore` | `containers/apptainer/lunarc/trim_galore.def` | `trim_galore --version` | `trim_galore --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `trimmomatic` | `containers/apptainer/lunarc/trimmomatic.def` | `trimmomatic --version` | `trimmomatic --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `umi_tools` | `containers/apptainer/lunarc/umi_tools.def` | `umi_tools --version` | `umi_tools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `verifybamid2` | `containers/apptainer/lunarc/verifybamid2.def` | `verifybamid2 --version` | `verifybamid2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `vsearch` | `containers/apptainer/lunarc/vsearch.def` | `vsearch --version` | `vsearch --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `yleaf` | `containers/apptainer/lunarc/yleaf.def` | `yleaf --version` | `yleaf --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |

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
| `adapterremoval` | `containers/apptainer/shared/adapterremoval.def` | `adapterremoval --version` | `adapterremoval --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `addeam` | `containers/apptainer/shared/addeam.def` | `addeam --version` | `addeam --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `alientrimmer` | `containers/apptainer/shared/alientrimmer.def` | `alientrimmer --version` | `alientrimmer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `angsd` | `containers/apptainer/shared/angsd.def` | `angsd 2>&1 | head -n 1` | `angsd -h` | `angsd -h` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `planned` |
| `atropos` | `containers/apptainer/shared/atropos.def` | `atropos --version` | `atropos trim --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `authenticct` | `containers/apptainer/shared/authenticct.def` | `authenticct --version` | `authenticct --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bamtools` | `containers/apptainer/shared/bamtools.def` | `bamtools --version` | `bamtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bayeshammer` | `containers/apptainer/shared/bayeshammer.def` | `bayeshammer --version` | `bayeshammer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bbduk` | `containers/apptainer/shared/bbduk.def` | `bbduk --version` | `bbduk --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bbmerge` | `containers/apptainer/shared/bbmerge.def` | `bbmerge --version` | `bbmerge --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bcftools` | `containers/apptainer/shared/bcftools.def` | `bcftools --version` | `bcftools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `beagle` | `containers/apptainer/shared/beagle.def` | `beagle --version` | `beagle --help` | `beagle --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `bedtools` | `containers/apptainer/shared/bedtools.def` | `bedtools --version` | `bedtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bowtie2` | `containers/apptainer/shared/bowtie2.def` | `bowtie2 --version` | `bowtie2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bowtie2_build` | `containers/apptainer/shared/bowtie2_build.def` | `bowtie2-build --version` | `bowtie2-build --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `bwa` | `containers/apptainer/shared/bwa.def` | `bwa --version` | `bwa --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `centrifuge` | `containers/apptainer/shared/centrifuge.def` | `centrifuge --version` | `centrifuge --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `clumpify` | `containers/apptainer/shared/clumpify.def` | `clumpify --version` | `clumpify --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `contammix` | `containers/apptainer/shared/contammix.def` | `contammix --version` | `contammix --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `cutadapt` | `containers/apptainer/shared/cutadapt.def` | `cutadapt --version` | `cutadapt --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `dada2` | `containers/apptainer/shared/dada2.def` | `dada2 --version` | `dada2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `damageprofiler` | `containers/apptainer/shared/damageprofiler.def` | `damageprofiler --version` | `damageprofiler --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `eagle` | `containers/apptainer/shared/eagle.def` | `eagle --version` | `eagle --help` | `eagle --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `experimental` |
| `eigensoft` | `containers/apptainer/shared/eigensoft.def` | `smartpca --version` | `smartpca --help` | `smartpca --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `fastp` | `containers/apptainer/shared/fastp.def` | `fastp --version` | `fastp --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `fastq_scan` | `containers/apptainer/shared/fastq_scan.def` | `fastq_scan --version` | `fastq_scan --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `fastqc` | `containers/apptainer/shared/fastqc.def` | `fastqc --version` | `fastqc --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `fastqvalidator` | `containers/apptainer/shared/fastqvalidator.def` | `fastqvalidator --version` | `fastqvalidator --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `fastuniq` | `containers/apptainer/shared/fastuniq.def` | `fastuniq --version` | `fastuniq -h` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `fastx_clipper` | `containers/apptainer/shared/fastx_clipper.def` | `fastx_clipper --version` | `fastx_clipper --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `flash2` | `containers/apptainer/shared/flash2.def` | `flash2 --version` | `flash2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `fqtools` | `containers/apptainer/shared/fqtools.def` | `fqtools --version` | `fqtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `germline` | `containers/apptainer/shared/germline.def` | `germline --version` | `germline --help` | `germline --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `glimpse` | `containers/apptainer/shared/glimpse.def` | `glimpse --version` | `glimpse --help` | `glimpse --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `planned` |
| `ibdhap` | `containers/apptainer/shared/ibdhap.def` | `ibdhap --version` | `ibdhap --help` | `ibdhap --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `planned` |
| `ibdne` | `containers/apptainer/shared/ibdne.def` | `ibdne --version` | `ibdne --help` | `ibdne --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `planned` |
| `ibdseq` | `-` | `ibdseq --version` | `ibdseq --help` | `ibdseq --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `planned` |
| `impute5` | `containers/apptainer/shared/impute5.def` | `impute5 --version` | `impute5 --help` | `impute5 --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `planned` |
| `kaiju` | `containers/apptainer/shared/kaiju.def` | `kaiju --version` | `kaiju --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `king` | `containers/apptainer/shared/king.def` | `king --version` | `king --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `kraken2` | `containers/apptainer/shared/kraken2.def` | `kraken2 --version` | `kraken2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `krakenuniq` | `containers/apptainer/shared/krakenuniq.def` | `krakenuniq --version` | `krakenuniq --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `leehom` | `containers/apptainer/shared/leehom.def` | `leehom --version` | `leehom --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `lighter` | `containers/apptainer/shared/lighter.def` | `lighter --version` | `lighter --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `mapdamage2` | `containers/apptainer/shared/mapdamage2.def` | `mapdamage2 --version` | `mapdamage2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `minimac4` | `containers/apptainer/shared/minimac4.def` | `minimac4 --version` | `minimac4 --help` | `minimac4 --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `planned` |
| `mosdepth` | `containers/apptainer/shared/mosdepth.def` | `mosdepth --version` | `mosdepth --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `multiqc` | `containers/apptainer/shared/multiqc.def` | `multiqc --version` | `multiqc --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `musket` | `containers/apptainer/shared/musket.def` | `musket --version` | `musket --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `pear` | `containers/apptainer/shared/pear.def` | `pear --version` | `pear --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `plink` | `containers/apptainer/shared/plink.def` | `plink --version` | `plink --help` | `tmp=$(mktemp -d); printf 'FAM1 S1 0 0 1 1\\n' > \"$tmp/tiny.ped\"; printf '1 rs1 0 1000 A G\\n' > \"$tmp/tiny.map\"; plink --file \"$tmp/tiny\" --freq --allow-no-sex --out \"$tmp/out\" >/dev/null 2>&1; test -s \"$tmp/out.frq\"` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `plink2` | `containers/apptainer/shared/plink2.def` | `plink2 --version` | `plink2 --help` | `tmp=$(mktemp -d); printf 'FAM1 S1 0 0 1 1\\n' > \"$tmp/tiny.ped\"; printf '1 rs1 0 1000 A G\\n' > \"$tmp/tiny.map\"; plink2 --pedmap \"$tmp/tiny\" --freq --allow-no-sex --out \"$tmp/out\" >/dev/null 2>&1; test -s \"$tmp/out.afreq\"` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |
| `pmdtools` | `containers/apptainer/shared/pmdtools.def` | `pmdtools --version` | `pmdtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `prinseq` | `containers/apptainer/shared/prinseq.def` | `prinseq++ --version` | `prinseq++ --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `pydamage` | `containers/apptainer/shared/pydamage.def` | `pydamage --version` | `pydamage --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `rcorrector` | `containers/apptainer/shared/rcorrector.def` | `rcorrector --version` | `rcorrector --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `rxy` | `containers/apptainer/shared/rxy.def` | `rxy --version` | `rxy --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `samtools` | `containers/apptainer/shared/samtools.def` | `samtools --version` | `samtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `schmutzi` | `containers/apptainer/shared/schmutzi.def` | `schmutzi --version` | `schmutzi --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `seqkit` | `containers/apptainer/shared/seqkit.def` | `seqkit --version` | `seqkit --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `seqkit_stats` | `containers/apptainer/shared/seqkit_stats.def` | `seqkit_stats --version` | `seqkit_stats --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `seqtk` | `containers/apptainer/shared/seqtk.def` | `seqtk --version` | `seqtk --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `shapeit` | `-` | `shapeit --version` | `shapeit --help` | `shapeit --help` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `planned` |
| `shapeit5` | `containers/apptainer/shared/shapeit5.def` | `shapeit5 --version` | `shapeit5 --help` | `shapeit5 --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `experimental` |
| `skewer` | `containers/apptainer/shared/skewer.def` | `skewer --version` | `skewer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `sortmerna` | `containers/apptainer/shared/sortmerna.def` | `sortmerna --version` | `sortmerna --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `star` | `containers/apptainer/shared/star.def` | `star --version` | `star --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `trim_galore` | `containers/apptainer/shared/trim_galore.def` | `trim_galore --version` | `trim_galore --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `trimmomatic` | `containers/apptainer/shared/trimmomatic.def` | `trimmomatic --version` | `trimmomatic --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `umi_tools` | `containers/apptainer/shared/umi_tools.def` | `umi_tools --version` | `umi_tools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `verifybamid2` | `containers/apptainer/shared/verifybamid2.def` | `verifybamid2 --version` | `verifybamid2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `vsearch` | `containers/apptainer/shared/vsearch.def` | `vsearch --version` | `vsearch --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `production` |
| `yleaf` | `containers/apptainer/shared/yleaf.def` | `yleaf --version` | `yleaf --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `experimental` |

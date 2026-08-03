<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: cargo run -p bijux-dna-dev -- containers run generate-qa-matrix -->

# APPTAINER_QA_MATRIX

## Purpose
Generated matrix for Apptainer-enabled tools, runtime checks, and evidence qualification.

## Scope
Runtime commands come from the tool registries, support disposition comes from the canonical catalog, and qualification comes from the generated evidence report.

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
- Tool row exists iff registry runtimes include `apptainer`.
- `apptainer_def` and smoke command fields are surfaced for QA checks.
- Support disposition describes whether Bijux retains the tool; qualification is derived independently from immutable evidence.

| Tool ID | Apptainer Def | Smoke Version | Smoke Help | Smoke Minimal | Minimal Exit | Docker Digest | Apptainer Digest | Minimal Rationale | QA Rule | Support | Qualification |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `adapterremoval` | `containers/apptainer/adapterremoval.def` | `adapterremoval --version` | `adapterremoval --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `addeam` | `containers/apptainer/addeam.def` | `addeam --version` | `addeam --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `alientrimmer` | `containers/apptainer/alientrimmer.def` | `alientrimmer --version` | `alientrimmer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `angsd` | `containers/apptainer/angsd.def` | `angsd 2>&1 | head -n 1` | `angsd -h` | `angsd -h` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version only until low-coverage GL fixtures and governed minimal runs are committed.` | `build+smoke required` | `retained` | `unverified` |
| `atropos` | `containers/apptainer/atropos.def` | `atropos --version` | `atropos trim --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `authenticct` | `containers/apptainer/authenticct.def` | `authenticct --version` | `authenticct --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `bamtools` | `containers/apptainer/bamtools.def` | `bamtools --version` | `bamtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `bamutil` | `containers/apptainer/bamutil.def` | `bamutil --version` | `bamutil --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `bayeshammer` | `containers/apptainer/bayeshammer.def` | `bayeshammer --version` | `bayeshammer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `bbduk` | `containers/apptainer/bbduk.def` | `bbduk --version` | `bbduk --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `bbmerge` | `containers/apptainer/bbmerge.def` | `bbmerge --version` | `bbmerge --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `bcftools` | `containers/apptainer/bcftools.def` | `bcftools --version` | `bcftools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `beagle` | `containers/apptainer/beagle.def` | `beagle --version` | `beagle --help` | `beagle --help` | `0` | `-` | `-` | `no deterministic phasing fixture is committed yet; keep help/version smoke until a governed minimal run exists.` | `build+smoke required` | `retained` | `unverified` |
| `bedtools` | `containers/apptainer/bedtools.def` | `bedtools --version` | `bedtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `bijux_dna` | `containers/apptainer/bijux_dna.def` | `bijux-dna --version` | `bijux-dna --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `bowtie2` | `containers/apptainer/bowtie2.def` | `bowtie2 --version` | `bowtie2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `bowtie2_build` | `containers/apptainer/bowtie2_build.def` | `bowtie2-build --version` | `bowtie2-build --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `bracken` | `containers/apptainer/bracken.def` | `bracken --version` | `bracken -h` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `bwa` | `containers/apptainer/bwa.def` | `bwa --version` | `bwa --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `centrifuge` | `containers/apptainer/centrifuge.def` | `centrifuge --version` | `centrifuge --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `clumpify` | `containers/apptainer/clumpify.def` | `clumpify --version` | `clumpify --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `contammix` | `containers/apptainer/contammix.def` | `contammix --version` | `contammix --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `cutadapt` | `containers/apptainer/cutadapt.def` | `cutadapt --version` | `cutadapt --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `dada2` | `containers/apptainer/dada2.def` | `dada2 --version` | `dada2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `damageprofiler` | `containers/apptainer/damageprofiler.def` | `damageprofiler --version` | `damageprofiler --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `diamond` | `containers/apptainer/diamond.def` | `diamond --version` | `diamond help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `dustmasker` | `containers/apptainer/dustmasker.def` | `dustmasker --version` | `dustmasker --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `eagle` | `containers/apptainer/eagle.def` | `eagle --version` | `eagle --help` | `eagle --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `retained` | `unverified` |
| `eigensoft` | `containers/apptainer/eigensoft.def` | `smartpca --version` | `smartpca --help` | `smartpca --help` | `0` | `-` | `-` | `no deterministic PCA fixture is committed yet; keep help/version smoke until a governed minimal run exists.` | `build+smoke required` | `retained` | `unverified` |
| `fastp` | `containers/apptainer/fastp.def` | `fastp --version` | `fastp --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `fastq_scan` | `containers/apptainer/fastq_scan.def` | `fastq_scan --version` | `fastq_scan --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `fastq_screen` | `containers/apptainer/fastq_screen.def` | `fastq_screen --version` | `fastq_screen --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `fastqc` | `containers/apptainer/fastqc.def` | `fastqc --version` | `fastqc --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `fastqvalidator` | `containers/apptainer/fastqvalidator.def` | `fastqvalidator --version` | `fastqvalidator --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `fastx_clipper` | `containers/apptainer/fastx_clipper.def` | `fastx_clipper --version` | `fastx_clipper --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `flash2` | `containers/apptainer/flash2.def` | `flash2 --version` | `flash2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `fqtools` | `containers/apptainer/fqtools.def` | `fqtools --version` | `fqtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `gatk` | `containers/apptainer/gatk.def` | `gatk --version` | `gatk --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `germline` | `containers/apptainer/germline.def` | `germline --version` | `germline --help` | `germline --help` | `0` | `-` | `-` | `no deterministic IBD segment fixture is committed yet; keep help/version smoke until a governed minimal run exists.` | `build+smoke required` | `retained` | `unverified` |
| `glimpse` | `containers/apptainer/glimpse.def` | `glimpse --version` | `glimpse --help` | `glimpse --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `retained` | `unverified` |
| `ibdhap` | `containers/apptainer/ibdhap.def` | `ibdhap --version` | `ibdhap --help` | `ibdhap --help` | `0` | `-` | `-` | `scientific execution requires a phased SNP panel and genetic map; the minimal gate verifies the stable adapter surface.` | `build+smoke required` | `retained` | `unverified` |
| `ibdne` | `containers/apptainer/ibdne.def` | `ibdne --version` | `ibdne --help` | `ibdne --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `retained` | `unverified` |
| `ibdseq` | `containers/apptainer/ibdseq.def` | `ibdseq --version` | `ibdseq --help` | `ibdseq --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper surface exposes help/version only until packaging and IBD fixtures are governed.` | `build+smoke required` | `retained` | `unverified` |
| `kaiju` | `containers/apptainer/kaiju.def` | `kaiju --version` | `kaiju --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `king` | `containers/apptainer/king.def` | `king --version` | `king --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `kraken2` | `containers/apptainer/kraken2.def` | `kraken2 --version` | `kraken2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `krakenuniq` | `containers/apptainer/krakenuniq.def` | `krakenuniq --version` | `krakenuniq --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `leehom` | `containers/apptainer/leehom.def` | `leehom --version` | `leehom --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `lighter` | `containers/apptainer/lighter.def` | `lighter --version` | `lighter --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `mapdamage2` | `containers/apptainer/mapdamage2.def` | `mapdamage2 --version` | `mapdamage2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `metaphlan` | `containers/apptainer/metaphlan.def` | `metaphlan --version` | `metaphlan --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `minimac4` | `containers/apptainer/minimac4.def` | `minimac4 --version` | `minimac4 --help` | `minimac4 --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper image exposes help/version contract only.` | `build+smoke required` | `retained` | `unverified` |
| `mosdepth` | `containers/apptainer/mosdepth.def` | `mosdepth --version` | `mosdepth --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `multiqc` | `containers/apptainer/multiqc.def` | `multiqc --version` | `multiqc --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `musket` | `containers/apptainer/musket.def` | `musket --version` | `musket --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `ngsbriggs` | `containers/apptainer/ngsbriggs.def` | `ngsbriggs --version` | `ngsbriggs --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `pear` | `containers/apptainer/pear.def` | `pear --version` | `pear --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `picard` | `containers/apptainer/picard.def` | `picard --version` | `picard --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `plink` | `containers/apptainer/plink.def` | `plink --version` | `plink --help` | `tmp=$(mktemp -d); printf 'FAM1 S1 0 0 1 1\\n' > \"$tmp/tiny.ped\"; printf '1 rs1 0 1000 A G\\n' > \"$tmp/tiny.map\"; plink --file \"$tmp/tiny\" --freq --allow-no-sex --out \"$tmp/out\" >/dev/null 2>&1; test -s \"$tmp/out.frq\"` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `plink2` | `containers/apptainer/plink2.def` | `plink2 --version` | `plink2 --help` | `tmp=$(mktemp -d); printf 'FAM1 S1 0 0 1 1\\n' > \"$tmp/tiny.ped\"; printf '1 rs1 0 1000 A G\\n' > \"$tmp/tiny.map\"; plink2 --pedmap \"$tmp/tiny\" --freq --allow-no-sex --out \"$tmp/out\" >/dev/null 2>&1; test -s \"$tmp/out.afreq\"` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `pmdtools` | `containers/apptainer/pmdtools.def` | `pmdtools --version` | `pmdtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `preseq` | `containers/apptainer/preseq.def` | `preseq --version` | `preseq --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `prinseq` | `containers/apptainer/prinseq.def` | `prinseq++ --version` | `prinseq++ --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `pydamage` | `containers/apptainer/pydamage.def` | `pydamage --version` | `pydamage --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `qualimap` | `containers/apptainer/qualimap.def` | `qualimap --version` | `qualimap --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `rcorrector` | `containers/apptainer/rcorrector.def` | `rcorrector --version` | `rcorrector --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `rxy` | `containers/apptainer/rxy.def` | `rxy --version` | `rxy --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `samtools` | `containers/apptainer/samtools.def` | `samtools --version` | `samtools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `schmutzi` | `containers/apptainer/schmutzi.def` | `schmutzi --version` | `schmutzi --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `seqfu` | `containers/apptainer/seqfu.def` | `seqfu --version` | `seqfu --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `seqkit` | `containers/apptainer/seqkit.def` | `seqkit --version` | `seqkit --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `seqkit_stats` | `containers/apptainer/seqkit_stats.def` | `seqkit_stats --version` | `seqkit_stats --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `seqprep` | `containers/apptainer/seqprep.def` | `seqprep --version` | `seqprep --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `seqpurge` | `containers/apptainer/seqpurge.def` | `seqpurge --version` | `seqpurge --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `seqtk` | `containers/apptainer/seqtk.def` | `seqtk --version` | `seqtk --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `shapeit` | `containers/apptainer/shapeit.def` | `shapeit --version` | `shapeit --help` | `shapeit --help` | `0` | `-` | `-` | `no-run-possible: planned wrapper surface exposes help/version only until packaging and phasing fixtures are governed.` | `build+smoke required` | `retained` | `unverified` |
| `shapeit5` | `containers/apptainer/shapeit5.def` | `shapeit5 --version` | `shapeit5 --help` | `shapeit5 --help` | `0` | `-` | `-` | `The suite help command verifies dispatch to the compiled upstream command family.` | `build+smoke required` | `retained` | `unverified` |
| `skewer` | `containers/apptainer/skewer.def` | `skewer --version` | `skewer --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `sortmerna` | `containers/apptainer/sortmerna.def` | `sortmerna --version` | `sortmerna --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `spades` | `containers/apptainer/spades.def` | `spades.py --version` | `spades.py --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `star` | `containers/apptainer/star.def` | `star --version` | `star --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `trim_galore` | `containers/apptainer/trim_galore.def` | `trim_galore --version` | `trim_galore --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `trimmomatic` | `containers/apptainer/trimmomatic.def` | `trimmomatic --version` | `trimmomatic --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `umi_tools` | `containers/apptainer/umi_tools.def` | `umi_tools --version` | `umi_tools --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `verifybamid2` | `containers/apptainer/verifybamid2.def` | `verifybamid2 --version` | `verifybamid2 --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `vsearch` | `containers/apptainer/vsearch.def` | `vsearch --version` | `vsearch --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |
| `yleaf` | `containers/apptainer/yleaf.def` | `yleaf --version` | `yleaf --help` | `-` | `0` | `-` | `-` | `minimal command contract` | `build+smoke required` | `retained` | `unverified` |

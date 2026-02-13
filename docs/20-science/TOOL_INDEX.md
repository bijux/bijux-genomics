<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: scripts/tooling/generate-tool-index.sh -->

# TOOL_INDEX

## Purpose
Generated index of registry tools with stage bindings and container references/self-reports.

## Scope
Source of truth = registry contracts + `artifacts/containers/summary.json` self-reports when available.

## Non-goals
- Replacing full scientific method docs for each domain.

## Contracts
- Manual edits are forbidden; regenerate via script.
- Source of truth is registry + containers; this file is a rendered view.
- Tool admission policy is documented in `docs/50-reference/TOOL_ADMISSION.md`.

See also: [Tool Admission](../50-reference/TOOL_ADMISSION.md)
See also: [VCF Downstream Roadmap](vcf/ROADMAP.md)

## VCF Downstream / IBD Toolkit

- `bcftools` (production) : vcf.call, vcf.filter, vcf.stats
- `beagle` (experimental) : vcf.phasing
- `eagle` (experimental) : vcf.phasing
- `eigensoft` (experimental) : vcf.pca, vcf.population_structure
- `germline` (experimental) : vcf.ibd
- `glimpse` (planned) : vcf.impute, vcf.imputation
- `ibdhap` (planned) : vcf.ibd
- `ibdne` (planned) : vcf.ibd, vcf.demography
- `ibdseq` (planned) : vcf.ibd
- `impute5` (planned) : vcf.impute, vcf.imputation
- `minimac4` (planned) : vcf.impute, vcf.imputation
- `plink` (experimental) : vcf.qc, vcf.admixture
- `plink2` (experimental) : vcf.qc, vcf.pca, vcf.population_structure, vcf.roh, vcf.admixture
- `shapeit` (planned) : vcf.phasing
- `shapeit5` (experimental) : vcf.phasing

| Tool ID | Purpose | Stage Bindings | Container Ref | Version | Citation | Status |
|---|---|---|---|---|---|---|
| `adapterremoval` | `trimmer` | `fastq.trim` | `bijuxdna/adapterremoval@sha256:5b618834ce9fc6376c9605c3a69d738236b9be48fdf493c1bc0945568a50808d` | `latest-pinned` | upstream:https://github.com/MikkelSchubert/adapterremoval | `experimental` |
| `addeam` | `transform` | `bam.damage` | `bijuxdna/addeam:latest-pinned` | `latest-pinned` | upstream:https://github.com/LouisPwr/AdDeam | `experimental` |
| `alientrimmer` | `trimmer` | `fastq.trim` | `bijuxdna/alientrimmer:0.4.0` | `0.4.0` | upstream:https://gite.lirmm.fr/clegrand/AlienTrimmer | `production` |
| `angsd` | `transform` | `bam.sex, bam.kinship` | `bijuxdna/angsd:0.940` | `0.940` | upstream:https://github.com/ANGSD/angsd | `production` |
| `atropos` | `trimmer` | `fastq.trim` | `bijuxdna/atropos@sha256:5a2fab22811303ced83475111a2427eaa894fcb39436618bb26be1da391f300d` | `1.1.31` | upstream:https://github.com/jdidion/atropos | `experimental` |
| `authenticct` | `transform` | `bam.authenticity` | `bijuxdna/authenticct:1.0.0` | `1.0.0` | upstream:https://github.com/StephanePeyregne/AuthentiCT | `production` |
| `bamtools` | `filter` | `bam.validate, bam.filter, bam.mapq_filter` | `bijuxdna/bamtools:2.5.2` | `2.5.2` | upstream:https://github.com/pezmaster31/bamtools | `production` |
| `bayeshammer` | `corrector` | `fastq.correct` | `bijuxdna/bayeshammer@sha256:pending` | `latest-pinned` | upstream:https://github.com/ablab/spades | `experimental` |
| `bbduk` | `transform` | `fastq.trim, fastq.low_complexity` | `bijuxdna/bbduk@sha256:da5764715915a5edeb0e40e2c18a5ce7142f31dac8e4844bd2dcb463403b8bd4` | `39.08` | upstream:https://sourceforge.net/projects/bbmap/ | `production` |
| `bbmerge` | `merger` | `fastq.merge` | `bijuxdna/bbmerge@sha256:6182848b989c8dbf094e06c486190b5d54243ac8eea542daa2c5c059a11bba54` | `39.01` | upstream:https://sourceforge.net/projects/bbmap/ | `experimental` |
| `bcftools` | `unknown` | `vcf.call, vcf.filter, vcf.stats` | `quay.io/biocontainers/bcftools:1.20--h8b25389_0@sha256:67f54df47f501f6ddef08e3b9ad89cf693952f9a89de0d74df6e39fce15f1ff6` | `1.20` | DOI:10.1093/gigascience/giab008 | `production` |
| `beagle` | `phasing` | `vcf.phasing` | `registry_lock` | `5.4` | planned | `experimental` |
| `bedtools` | `filter` | `bam.validate, bam.filter` | `bijuxdna/bedtools:2.31.1` | `2.31.1` | upstream:https://github.com/arq5x/bedtools2 | `production` |
| `bowtie2` | `aligner` | `bam.align` | `bijuxdna/bowtie2:2.5.4` | `2.5.4` | upstream:https://github.com/BenLangmead/bowtie2 | `production` |
| `bracken` | `screen` | `fastq.screen` | `bijuxdna/bracken:2.9` | `2.9` | upstream:https://github.com/jenniferlu717/Bracken | `production` |
| `bwa` | `aligner` | `bam.align` | `bijuxdna/bwa:0.7.17` | `0.7.17` | upstream:https://github.com/lh3/bwa | `production` |
| `centrifuge` | `screen` | `fastq.screen` | `bijuxdna/centrifuge@sha256:pending` | `1.0.4` | upstream:https://github.com/DaehwanKimLab/centrifuge | `experimental` |
| `contammix` | `transform` | `bam.contamination` | `bijuxdna/contammix:1.0.11` | `1.0.11` | upstream:https://bioconductor.org/packages/contamMix | `production` |
| `cutadapt` | `trimmer` | `fastq.trim` | `bijuxdna/cutadapt@sha256:4405f2effc1a195c93098408aa36268357c25b758348bfe6da8790bbe7e842ba` | `latest-pinned` | upstream:https://github.com/cutadapt/cutadapt | `experimental` |
| `damageprofiler` | `transform` | `bam.damage, bam.authenticity` | `bijuxdna/damageprofiler:latest-pinned` | `latest-pinned` | upstream:https://github.com/Integrative-Transcriptomics/DamageProfiler | `experimental` |
| `eagle` | `phasing` | `vcf.phasing` | `registry_lock` | `5.4` | planned | `experimental` |
| `eigensoft` | `population_structure` | `vcf.pca, vcf.population_structure` | `registry_lock` | `8.0.0` | planned | `experimental` |
| `fastp` | `filter` | `fastq.trim, fastq.filter` | `bijuxdna/fastp@sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10` | `0.23.4` | upstream:https://github.com/OpenGene/fastp/archive/v${VERSION_FASTP}.tar.gz | `production` |
| `fastq.validate_pre` | `merger` | `fastq.merge` | `bijuxdna/vsearch@sha256:c16ef98d6fd67ac0b8eea3ebb4f3dc6df9c582d6f838317d5f6ccc7a09e60bb3` | `2.28.1` | upstream:https://github.com/vsearch/vsearch | `production` |
| `fastq_screen` | `screen` | `fastq.screen` | `bijuxdna/fastq_screen@sha256:pending` | `0.15.3` | upstream:https://github.com/fastq_screen/fastq_screen | `experimental` |
| `fastqc` | `trimmer` | `fastq.detect_adapters` | `bijuxdna/fastqc@sha256:e0b83c56262486cab51020e2bb809b391ad9b38ba7a898588ab15b73586ee789` | `0.12.1` | upstream:https://www.bioinformatics.babraham.ac.uk/projects/fastqc/fastqc_v${VERSION_FASTQC}.zip | `production` |
| `fastqvalidator` | `validator` | `fastq.validate_pre` | `bijuxdna/fastqvalidator@sha256:0000000000000000000000000000000000000000000000000000000000000000` | `v0.1.1` | upstream:https://github.com/fastqvalidator/fastqvalidator | `production` |
| `fastx_clipper` | `trimmer` | `fastq.trim` | `bijuxdna/fastx_clipper:0.0.14` | `0.0.14` | upstream:https://github.com/agordon/fastx_toolkit | `production` |
| `flash2` | `merger` | `fastq.merge` | `bijuxdna/flash2@sha256:e3dfc866d56d1ca6d62c58ade5981e0b00fc3c8bf8148ecbd196ab56293e1dd5` | `2.2.00` | upstream:https://github.com/dstreett/FLASH2 | `experimental` |
| `fqtools` | `validator` | `fastq.validate_pre` | `bijuxdna/fqtools@sha256:0000000000000000000000000000000000000000000000000000000000000000` | `v2.3` | upstream:https://github.com/alastair-droop/fqtools | `experimental` |
| `germline` | `relatedness` | `vcf.ibd` | `registry_lock` | `1.5.3` | planned | `experimental` |
| `glimpse` | `imputation` | `vcf.impute, vcf.imputation` | `registry_lock` | `0.0.0-planned` | planned | `planned` |
| `ibdhap` | `relatedness` | `vcf.ibd` | `registry_lock` | `0.1.0-planned` | planned | `planned` |
| `ibdne` | `demography` | `vcf.ibd, vcf.demography` | `registry_lock` | `1.0-planned` | planned | `planned` |
| `ibdseq` | `relatedness` | `vcf.ibd` | `planned` | `0.0.0-planned` | planned | `planned` |
| `impute5` | `imputation` | `vcf.impute, vcf.imputation` | `registry_lock` | `0.0.0-planned` | planned | `planned` |
| `kaiju` | `screen` | `fastq.screen` | `bijuxdna/kaiju@sha256:pending` | `1.10.0` | upstream:https://github.com/bioinformatics-centre/kaiju | `experimental` |
| `king` | `transform` | `bam.kinship` | `bijuxdna/king:2.3.0` | `2.3.0` | upstream:https://www.kingrelatedness.com/ | `production` |
| `kraken2` | `screen` | `fastq.screen` | `bijuxdna/kraken2@sha256:pending` | `2.1.3` | upstream:https://github.com/DerrickWood/kraken2/archive/v${VERSION_KRAKEN2}.tar.gz | `production` |
| `krakenuniq` | `screen` | `fastq.screen` | `bijuxdna/krakenuniq:1.0.4` | `1.0.4` | upstream:https://github.com/fbreitwieser/krakenuniq | `production` |
| `leehom` | `merger` | `fastq.trim, fastq.merge` | `bijuxdna/leehom:latest-pinned` | `latest-pinned` | upstream:https://github.com/grenaud/leeHom | `experimental` |
| `lighter` | `corrector` | `fastq.correct` | `bijuxdna/lighter@sha256:pending` | `latest-pinned` | upstream:https://github.com/mourisl/Lighter | `experimental` |
| `mapdamage2` | `transform` | `bam.damage` | `bijuxdna/mapdamage2:2.2.2` | `2.2.2` | upstream:https://github.com/ginolhac/mapDamage | `production` |
| `metaphlan` | `screen` | `fastq.screen` | `bijuxdna/metaphlan@sha256:pending` | `4.1.1` | upstream:https://github.com/biobakery/MetaPhlAn | `experimental` |
| `minimac4` | `imputation` | `vcf.impute, vcf.imputation` | `registry_lock` | `0.0.0-planned` | planned | `planned` |
| `mosdepth` | `transform` | `bam.coverage` | `bijuxdna/mosdepth:0.3.10` | `0.3.10` | upstream:https://github.com/brentp/mosdepth | `production` |
| `multiqc` | `qc` | `fastq.qc_post` | `bijuxdna/multiqc@sha256:40af0025fcc5bc4ea15e5cd2a4fd7bcfc98ea06c9ca781e6268f3c81d12787ec` | `1.24` | upstream:https://github.com/multiqc/multiqc | `production` |
| `musket` | `corrector` | `fastq.correct` | `bijuxdna/musket@sha256:pending` | `1.1` | upstream:https://github.com/alexdobin/musket | `experimental` |
| `pear` | `merger` | `fastq.merge` | `bijuxdna/pear@sha256:4e00e9ffabc5ed46115efab5b6bae946913f1713bb314fd4acb7c379c37efae6` | `0.9.6` | upstream:https://github.com/xflouris/PEAR | `production` |
| `plink` | `qc_admixture` | `vcf.qc, vcf.admixture` | `registry_lock` | `1.90` | planned | `experimental` |
| `plink2` | `analysis` | `vcf.qc, vcf.pca, vcf.population_structure, vcf.roh, vcf.admixture` | `registry_lock` | `2.00a5` | planned | `experimental` |
| `pmdtools` | `transform` | `bam.damage, bam.authenticity` | `bijuxdna/pmdtools:0.60` | `0.60` | upstream:https://github.com/pontussk/PMDtools | `production` |
| `prinseq` | `filter` | `fastq.filter` | `bijuxdna/prinseq@sha256:7216ffecd7913edaea33ec76b3775ab0cb0d60064f31e96c63e043d578a3f971` | `1.2.4` | upstream:https://github.com/uwb-linux/prinseq | `experimental` |
| `pydamage` | `transform` | `bam.damage` | `bijuxdna/pydamage:1.0.0` | `1.0.0` | upstream:https://github.com/maxibor/pydamage | `production` |
| `qualimap` | `qc` | `fastq.qc_post` | `bijuxdna/qualimap@sha256:pending` | `2.3` | upstream:http://qualimap.conesalab.org/ | `experimental` |
| `rcorrector` | `corrector` | `fastq.correct` | `bijuxdna/rcorrector@sha256:pending` | `1.0.7` | upstream:https://github.com/mourisl/Rcorrector | `production` |
| `rxy` | `transform` | `bam.sex` | `bijuxdna/rxy:1.0.0` | `1.0.0` | upstream:https://github.com/bijux/bijux-dna | `production` |
| `samtools` | `aligner` | `bam.align, bam.validate, bam.qc_pre, bam.mapping_summary, bam.filter, bam.mapq_filter, bam.length_filter, bam.markdup, bam.duplication_metrics, bam.coverage, bam.endogenous_content, bam.overlap_correction, fastq.prepare_reference, fastq.qc_post` | `bijuxdna/samtools:1.21` | `1.21` | upstream:https://github.com/samtools/samtools | `production` |
| `schmutzi` | `transform` | `bam.contamination` | `bijuxdna/schmutzi:1.5.4` | `1.5.4` | upstream:https://github.com/grenaud/schmutzi | `production` |
| `seqkit` | `filter` | `fastq.filter` | `bijuxdna/seqkit@sha256:ca3dc13e3fef5d34927c44b2d8cd2bc6708c2c256f42e51369d7b1203b0d2991` | `2.8.2` | upstream:https://github.com/shenwei356/seqkit/releases/download/v${VERSION_SEQKIT}/seqkit_linux_arm64.tar.gz | `production` |
| `seqkit_stats` | `qc` | `fastq.stats_neutral` | `bijuxdna/seqkit@sha256:ca3dc13e3fef5d34927c44b2d8cd2bc6708c2c256f42e51369d7b1203b0d2991` | `2.7.0` | upstream:https://github.com/seqkit_stats/seqkit_stats | `production` |
| `seqtk` | `validator` | `fastq.validate_pre` | `bijuxdna/seqtk@sha256:0000000000000000000000000000000000000000000000000000000000000000` | `1.5-r133` | upstream:https://github.com/lh3/seqtk.git | `experimental` |
| `shapeit` | `phasing` | `vcf.phasing` | `planned` | `0.0.0-planned` | planned | `planned` |
| `shapeit5` | `phasing` | `vcf.phasing` | `registry_lock` | `5.4` | planned | `experimental` |
| `skewer` | `trimmer` | `fastq.trim` | `bijuxdna/skewer:latest-pinned` | `latest-pinned` | upstream:https://github.com/relipmoc/skewer | `experimental` |
| `sortmerna` | `filter` | `fastq.filter` | `bijuxdna/sortmerna:4.3.7` | `4.3.7` | upstream:https://github.com/biocore/sortmerna | `production` |
| `spades` | `corrector` | `fastq.correct` | `bijuxdna/spades@sha256:pending` | `latest-pinned` | upstream:https://github.com/ablab/spades | `experimental` |
| `star` | `transform` | `fastq.prepare_reference` | `bijuxdna/star:2.7.11b` | `2.7.11b` | upstream:https://github.com/alexdobin/STAR | `production` |
| `trim_galore` | `trimmer` | `fastq.trim` | `bijuxdna/trim_galore@sha256:f323405a5a0ba19bbdae765dd4269e9156c415993977996079b63c5eb5bb0a61` | `0.6.10` | upstream:https://github.com/FelixKrueger/TrimGalore | `experimental` |
| `trimmomatic` | `trimmer` | `fastq.trim` | `bijuxdna/trimmomatic@sha256:41c0d161444ee7bb6b36ead3bbceb998af611be6ead6784231c5440e092bd5a4` | `0.39` | upstream:http://www.usadellab.org/cms/uploads/supplementary/Trimmomatic/Trimmomatic-${VERSION_TRIMMOMATIC}.zip | `experimental` |
| `umi_tools` | `transform` | `fastq.umi` | `bijuxdna/umi_tools@sha256:pending` | `1.1.6` | upstream:https://github.com/umi_tools/umi_tools | `production` |
| `verifybamid2` | `transform` | `bam.contamination` | `bijuxdna/verifybamid2:2.0.1` | `2.0.1` | upstream:https://github.com/Griffan/VerifyBamID | `production` |
| `yleaf` | `transform` | `bam.sex, bam.haplogroups` | `bijuxdna/yleaf:latest-pinned` | `latest-pinned` | upstream:https://github.com/genid/Yleaf | `experimental` |

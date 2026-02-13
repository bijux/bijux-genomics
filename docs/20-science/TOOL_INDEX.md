<!-- GENERATED FILE - DO NOT EDIT -->
<!-- Regenerate with: scripts/tooling/generate-tool-index.sh -->

# TOOL_INDEX

## Purpose
Generated index of registry tools with stage bindings and container references.

## Scope
Derived from `configs/ci/registry/tool_registry*.toml`.

## Non-goals
- Replacing full scientific method docs for each domain.

## Contracts
- Manual edits are forbidden; regenerate via script.
- Tool admission policy is documented in `docs/50-reference/TOOL_ADMISSION.md`.

See also: [Tool Admission](../50-reference/TOOL_ADMISSION.md)

| Tool ID | Purpose | Stage Bindings | Container Ref | Citation | Status |
|---|---|---|---|---|---|
| `adapterremoval` | `trimmer` | `fastq.trim` | `bijuxdna/adapterremoval@sha256:5b618834ce9fc6376c9605c3a69d738236b9be48fdf493c1bc0945568a50808d` | upstream:https://github.com/MikkelSchubert/adapterremoval | `supported` |
| `addeam` | `transform` | `bam.damage` | `bijuxdna/addeam:latest-pinned` | upstream:https://github.com/LouisPwr/AdDeam | `supported` |
| `alientrimmer` | `trimmer` | `fastq.trim` | `bijuxdna/alientrimmer:0.4.0` | upstream:https://gite.lirmm.fr/clegrand/AlienTrimmer | `supported` |
| `angsd` | `transform` | `bam.sex, bam.kinship` | `bijuxdna/angsd:0.940` | upstream:https://github.com/ANGSD/angsd | `supported` |
| `atropos` | `trimmer` | `fastq.trim` | `bijuxdna/atropos@sha256:5a2fab22811303ced83475111a2427eaa894fcb39436618bb26be1da391f300d` | upstream:https://github.com/jdidion/atropos | `supported` |
| `authenticct` | `transform` | `bam.authenticity` | `bijuxdna/authenticct:1.0.0` | upstream:https://github.com/StephanePeyregne/AuthentiCT | `supported` |
| `bamtools` | `filter` | `bam.validate, bam.filter, bam.mapq_filter` | `bijuxdna/bamtools:2.5.2` | upstream:https://github.com/pezmaster31/bamtools | `supported` |
| `bayeshammer` | `corrector` | `fastq.correct` | `bijuxdna/bayeshammer@sha256:pending` | upstream:https://github.com/ablab/spades | `supported` |
| `bbduk` | `transform` | `fastq.trim, fastq.low_complexity` | `bijuxdna/bbduk@sha256:da5764715915a5edeb0e40e2c18a5ce7142f31dac8e4844bd2dcb463403b8bd4` | upstream:https://sourceforge.net/projects/bbmap/ | `supported` |
| `bbmerge` | `merger` | `fastq.merge` | `bijuxdna/bbmerge@sha256:6182848b989c8dbf094e06c486190b5d54243ac8eea542daa2c5c059a11bba54` | upstream:https://sourceforge.net/projects/bbmap/ | `supported` |
| `bcftools` | `unknown` | `vcf.call, vcf.filter, vcf.stats` | `quay.io/biocontainers/bcftools:1.20--h8b25389_0@sha256:67f54df47f501f6ddef08e3b9ad89cf693952f9a89de0d74df6e39fce15f1ff6` | DOI:10.1093/gigascience/giab008 | `supported` |
| `bedtools` | `filter` | `bam.validate, bam.filter` | `bijuxdna/bedtools:2.31.1` | upstream:https://github.com/arq5x/bedtools2 | `supported` |
| `bowtie2` | `aligner` | `bam.align` | `bijuxdna/bowtie2:2.5.4` | upstream:https://github.com/BenLangmead/bowtie2 | `supported` |
| `bracken` | `screen` | `fastq.screen` | `bijuxdna/bracken:2.9` | upstream:https://github.com/jenniferlu717/Bracken | `supported` |
| `bwa` | `aligner` | `bam.align` | `bijuxdna/bwa:0.7.17` | upstream:https://github.com/lh3/bwa | `supported` |
| `centrifuge` | `screen` | `fastq.screen` | `bijuxdna/centrifuge@sha256:pending` | upstream:https://github.com/DaehwanKimLab/centrifuge | `supported` |
| `contammix` | `transform` | `bam.contamination` | `bijuxdna/contammix:1.0.11` | upstream:https://bioconductor.org/packages/contamMix | `supported` |
| `cutadapt` | `trimmer` | `fastq.trim` | `bijuxdna/cutadapt@sha256:4405f2effc1a195c93098408aa36268357c25b758348bfe6da8790bbe7e842ba` | upstream:https://github.com/cutadapt/cutadapt | `supported` |
| `damageprofiler` | `transform` | `bam.damage, bam.authenticity` | `bijuxdna/damageprofiler:latest-pinned` | upstream:https://github.com/Integrative-Transcriptomics/DamageProfiler | `supported` |
| `fastp` | `filter` | `fastq.trim, fastq.filter` | `bijuxdna/fastp@sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10` | upstream:https://github.com/OpenGene/fastp/archive/v${VERSION_FASTP}.tar.gz | `supported` |
| `fastq.validate_pre` | `merger` | `fastq.merge` | `bijuxdna/vsearch@sha256:c16ef98d6fd67ac0b8eea3ebb4f3dc6df9c582d6f838317d5f6ccc7a09e60bb3` | upstream:https://github.com/vsearch/vsearch | `supported` |
| `fastq_screen` | `screen` | `fastq.screen` | `bijuxdna/fastq_screen@sha256:pending` | upstream:https://github.com/fastq_screen/fastq_screen | `supported` |
| `fastqc` | `trimmer` | `fastq.detect_adapters` | `bijuxdna/fastqc@sha256:e0b83c56262486cab51020e2bb809b391ad9b38ba7a898588ab15b73586ee789` | upstream:https://www.bioinformatics.babraham.ac.uk/projects/fastqc/fastqc_v${VERSION_FASTQC}.zip | `supported` |
| `fastqvalidator` | `validator` | `fastq.validate_pre` | `bijuxdna/fastqvalidator@sha256:0000000000000000000000000000000000000000000000000000000000000000` | upstream:https://github.com/fastqvalidator/fastqvalidator | `supported` |
| `fastx_clipper` | `trimmer` | `fastq.trim` | `bijuxdna/fastx_clipper:0.0.14` | upstream:https://github.com/agordon/fastx_toolkit | `supported` |
| `flash2` | `merger` | `fastq.merge` | `bijuxdna/flash2@sha256:e3dfc866d56d1ca6d62c58ade5981e0b00fc3c8bf8148ecbd196ab56293e1dd5` | upstream:https://github.com/dstreett/FLASH2 | `supported` |
| `fqtools` | `validator` | `fastq.validate_pre` | `bijuxdna/fqtools@sha256:0000000000000000000000000000000000000000000000000000000000000000` | upstream:https://github.com/alastair-droop/fqtools | `supported` |
| `kaiju` | `screen` | `fastq.screen` | `bijuxdna/kaiju@sha256:pending` | upstream:https://github.com/bioinformatics-centre/kaiju | `supported` |
| `king` | `transform` | `bam.kinship` | `bijuxdna/king:2.3.0` | upstream:https://www.kingrelatedness.com/ | `supported` |
| `kraken2` | `screen` | `fastq.screen` | `bijuxdna/kraken2@sha256:pending` | upstream:https://github.com/DerrickWood/kraken2/archive/v${VERSION_KRAKEN2}.tar.gz | `supported` |
| `krakenuniq` | `screen` | `fastq.screen` | `bijuxdna/krakenuniq:1.0.4` | upstream:https://github.com/fbreitwieser/krakenuniq | `supported` |
| `leehom` | `merger` | `fastq.trim, fastq.merge` | `bijuxdna/leehom:latest-pinned` | upstream:https://github.com/grenaud/leeHom | `supported` |
| `lighter` | `corrector` | `fastq.correct` | `bijuxdna/lighter@sha256:pending` | upstream:https://github.com/mourisl/Lighter | `supported` |
| `mapdamage2` | `transform` | `bam.damage` | `bijuxdna/mapdamage2:2.2.2` | upstream:https://github.com/ginolhac/mapDamage | `supported` |
| `metaphlan` | `screen` | `fastq.screen` | `bijuxdna/metaphlan@sha256:pending` | upstream:https://github.com/biobakery/MetaPhlAn | `supported` |
| `mosdepth` | `transform` | `bam.coverage` | `bijuxdna/mosdepth:0.3.10` | upstream:https://github.com/brentp/mosdepth | `supported` |
| `multiqc` | `qc` | `fastq.qc_post` | `bijuxdna/multiqc@sha256:40af0025fcc5bc4ea15e5cd2a4fd7bcfc98ea06c9ca781e6268f3c81d12787ec` | upstream:https://github.com/multiqc/multiqc | `supported` |
| `musket` | `corrector` | `fastq.correct` | `bijuxdna/musket@sha256:pending` | upstream:https://github.com/alexdobin/musket | `supported` |
| `pear` | `merger` | `fastq.merge` | `bijuxdna/pear@sha256:4e00e9ffabc5ed46115efab5b6bae946913f1713bb314fd4acb7c379c37efae6` | upstream:https://github.com/xflouris/PEAR | `supported` |
| `pmdtools` | `transform` | `bam.damage, bam.authenticity` | `bijuxdna/pmdtools:0.60` | upstream:https://github.com/pontussk/PMDtools | `supported` |
| `prinseq` | `filter` | `fastq.filter` | `bijuxdna/prinseq@sha256:7216ffecd7913edaea33ec76b3775ab0cb0d60064f31e96c63e043d578a3f971` | upstream:https://github.com/uwb-linux/prinseq | `supported` |
| `pydamage` | `transform` | `bam.damage` | `bijuxdna/pydamage:1.0.0` | upstream:https://github.com/maxibor/pydamage | `supported` |
| `qualimap` | `qc` | `fastq.qc_post` | `bijuxdna/qualimap@sha256:pending` | upstream:http://qualimap.conesalab.org/ | `supported` |
| `rcorrector` | `corrector` | `fastq.correct` | `bijuxdna/rcorrector@sha256:pending` | upstream:https://github.com/mourisl/Rcorrector | `supported` |
| `rxy` | `transform` | `bam.sex` | `bijuxdna/rxy:1.0.0` | upstream:https://github.com/bijux/bijux-dna | `supported` |
| `samtools` | `aligner` | `bam.align, bam.validate, bam.qc_pre, bam.mapping_summary, bam.filter, bam.mapq_filter, bam.length_filter, bam.markdup, bam.duplication_metrics, bam.coverage, bam.endogenous_content, bam.overlap_correction, fastq.prepare_reference, fastq.qc_post` | `bijuxdna/samtools:1.21` | upstream:https://github.com/samtools/samtools | `supported` |
| `schmutzi` | `transform` | `bam.contamination` | `bijuxdna/schmutzi:1.5.4` | upstream:https://github.com/grenaud/schmutzi | `supported` |
| `seqkit` | `filter` | `fastq.filter` | `bijuxdna/seqkit@sha256:ca3dc13e3fef5d34927c44b2d8cd2bc6708c2c256f42e51369d7b1203b0d2991` | upstream:https://github.com/shenwei356/seqkit/releases/download/v${VERSION_SEQKIT}/seqkit_linux_arm64.tar.gz | `supported` |
| `seqkit_stats` | `qc` | `fastq.stats_neutral` | `bijuxdna/seqkit@sha256:ca3dc13e3fef5d34927c44b2d8cd2bc6708c2c256f42e51369d7b1203b0d2991` | upstream:https://github.com/seqkit_stats/seqkit_stats | `supported` |
| `seqtk` | `validator` | `fastq.validate_pre` | `bijuxdna/seqtk@sha256:0000000000000000000000000000000000000000000000000000000000000000` | upstream:https://github.com/lh3/seqtk.git | `supported` |
| `skewer` | `trimmer` | `fastq.trim` | `bijuxdna/skewer:latest-pinned` | upstream:https://github.com/relipmoc/skewer | `supported` |
| `sortmerna` | `filter` | `fastq.filter` | `bijuxdna/sortmerna:4.3.7` | upstream:https://github.com/biocore/sortmerna | `supported` |
| `spades` | `corrector` | `fastq.correct` | `bijuxdna/spades@sha256:pending` | upstream:https://github.com/ablab/spades | `supported` |
| `star` | `transform` | `fastq.prepare_reference` | `bijuxdna/star:2.7.11b` | upstream:https://github.com/alexdobin/STAR | `supported` |
| `trim_galore` | `trimmer` | `fastq.trim` | `bijuxdna/trim_galore@sha256:f323405a5a0ba19bbdae765dd4269e9156c415993977996079b63c5eb5bb0a61` | upstream:https://github.com/FelixKrueger/TrimGalore | `supported` |
| `trimmomatic` | `trimmer` | `fastq.trim` | `bijuxdna/trimmomatic@sha256:41c0d161444ee7bb6b36ead3bbceb998af611be6ead6784231c5440e092bd5a4` | upstream:http://www.usadellab.org/cms/uploads/supplementary/Trimmomatic/Trimmomatic-${VERSION_TRIMMOMATIC}.zip | `supported` |
| `umi_tools` | `transform` | `fastq.umi` | `bijuxdna/umi_tools@sha256:pending` | upstream:https://github.com/umi_tools/umi_tools | `supported` |
| `verifybamid2` | `transform` | `bam.contamination` | `bijuxdna/verifybamid2:2.0.1` | upstream:https://github.com/Griffan/VerifyBamID | `supported` |
| `yleaf` | `transform` | `bam.sex, bam.haplogroups` | `bijuxdna/yleaf:latest-pinned` | upstream:https://github.com/genid/Yleaf | `supported` |

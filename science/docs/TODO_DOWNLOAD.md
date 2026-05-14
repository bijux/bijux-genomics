# Science Docs Download Backlog

This file tracks manual evidence acquisition for `science/docs/`.

## Purpose

This backlog reserves durable local archive paths for non-shareable evidence that
supports science review, container version validation, and tool provenance
checks.

## Stability

Keep identifiers and archive paths durable. Add new rows instead of renaming old
ones when the evidence set evolves.

## Status

The canonical machine-readable backlog is
[science/generated/current/evidence/fastq_download_backlog.tsv](../generated/current/evidence/fastq_download_backlog.tsv).

This markdown file is the operator-facing view of that FASTQ backlog.

GitHub repository evidence is governed separately through
[science/docs/upstream/README.md](upstream/README.md),
[science/docs/upstream/github-repos/README.md](upstream/github-repos/README.md),
and
[science/docs/upstream/github-repos/MANIFEST.tsv](upstream/github-repos/MANIFEST.tsv).

That manifest is the durable target list for local repository mirrors under
`science/docs/upstream/github-repos/mirrors/`.

Paper payloads are governed separately through
[science/docs/upstream/papers/README.md](upstream/papers/README.md)
and
[science/docs/upstream/papers/TODO_DOWNLOAD.md](upstream/papers/TODO_DOWNLOAD.md).

## Ready for Manual Clone

| Source ID | Tool | FASTQ Stages | Archive Path | Upstream |
| --- | --- | --- | --- | --- |
| `source.fastq.tool.adapterremoval.upstream` | `adapterremoval` | `fastq.merge_pairs,fastq.trim_reads,fastq.trim_terminal_damage` | `science/docs/upstream/fastq/tools/adapterremoval/repo` | https://github.com/MikkelSchubert/adapterremoval |
| `source.fastq.tool.alientrimmer.upstream` | `alientrimmer` | `fastq.trim_reads` | `science/docs/upstream/fastq/tools/alientrimmer/repo` | https://gitlab.pasteur.fr/GIPhy/AlienTrimmer |
| `source.fastq.tool.atropos.upstream` | `atropos` | `fastq.trim_reads` | `science/docs/upstream/fastq/tools/atropos/repo` | https://github.com/jdidion/atropos |
| `source.fastq.tool.bayeshammer.upstream` | `bayeshammer` | `fastq.correct_errors` | `science/docs/upstream/fastq/tools/bayeshammer/repo` | https://github.com/ablab/spades |
| `source.fastq.tool.bijux_dna.upstream` | `bijux_dna` | `fastq.build_contaminant_db,fastq.build_rrna_db,fastq.build_taxonomy_db,fastq.capture_provenance_snapshot,fastq.classify_layout,fastq.concatenate_lanes,fastq.deinterleave_reads,fastq.demultiplex_reads,fastq.detect_duplicates_premerge,fastq.detect_instrument_artifacts,fastq.estimate_library_complexity_prealign,fastq.interleave_reads,fastq.materialize_qc_manifest,fastq.normalize_read_names,fastq.prepare_adapter_bank,fastq.prepare_host_reference_bundle,fastq.prepare_primer_bank,fastq.repair_pairs,fastq.subsample_reads,fastq.verify_assets` | `science/docs/upstream/fastq/tools/bijux_dna/repo` | https://github.com/bijux/bijux-genomics |
| `source.fastq.tool.bowtie2.upstream` | `bowtie2` | `fastq.deplete_host,fastq.deplete_reference_contaminants` | `science/docs/upstream/fastq/tools/bowtie2/repo` | https://github.com/BenLangmead/bowtie2 |
| `source.fastq.tool.bowtie2_build.upstream` | `bowtie2_build` | `fastq.index_reference` | `science/docs/upstream/fastq/tools/bowtie2_build/repo` | https://github.com/BenLangmead/bowtie2 |
| `source.fastq.tool.centrifuge.upstream` | `centrifuge` | `fastq.screen_taxonomy` | `science/docs/upstream/fastq/tools/centrifuge/repo` | https://github.com/DaehwanKimLab/centrifuge |
| `source.fastq.tool.cutadapt.upstream` | `cutadapt` | `fastq.normalize_primers,fastq.trim_reads,fastq.trim_terminal_damage` | `science/docs/upstream/fastq/tools/cutadapt/repo` | https://github.com/marcelm/cutadapt |
| `source.fastq.tool.dada2.upstream` | `dada2` | `fastq.infer_asvs` | `science/docs/upstream/fastq/tools/dada2/repo` | https://github.com/benjjneb/dada2 |
| `source.fastq.tool.diamond.upstream` | `diamond` | `fastq.screen_taxonomy` | `science/docs/upstream/fastq/tools/diamond/repo` | https://github.com/bbuchfink/diamond |
| `source.fastq.tool.fastq_scan.upstream` | `fastq_scan` | `fastq.profile_overrepresented_sequences,fastq.validate_reads` | `science/docs/upstream/fastq/tools/fastq_scan/repo` | https://github.com/rpetit3/fastq-scan |
| `source.fastq.tool.fastqvalidator.upstream` | `fastqvalidator` | `fastq.validate_reads` | `science/docs/upstream/fastq/tools/fastqvalidator/repo` | https://github.com/statgen/fastQValidator |
| `source.fastq.tool.fastp.upstream` | `fastp` | `fastq.filter_low_complexity,fastq.filter_reads,fastq.profile_read_lengths,fastq.trim_polyg_tails,fastq.trim_reads` | `science/docs/upstream/fastq/tools/fastp/repo` | https://github.com/OpenGene/fastp |
| `source.fastq.tool.fastqc.upstream` | `fastqc` | `fastq.detect_adapters,fastq.profile_overrepresented_sequences,fastq.validate_reads` | `science/docs/upstream/fastq/tools/fastqc/repo` | https://github.com/s-andrews/FastQC |
| `source.fastq.tool.fastx_clipper.upstream` | `fastx_clipper` | `fastq.trim_reads` | `science/docs/upstream/fastq/tools/fastx_clipper/repo` | https://github.com/agordon/fastx_toolkit |
| `source.fastq.tool.flash2.upstream` | `flash2` | `fastq.merge_pairs` | `science/docs/upstream/fastq/tools/flash2/repo` | https://github.com/dstreett/FLASH2 |
| `source.fastq.tool.fqtools.upstream` | `fqtools` | `fastq.validate_reads` | `science/docs/upstream/fastq/tools/fqtools/repo` | https://github.com/alastair-droop/fqtools |
| `source.fastq.tool.kaiju.upstream` | `kaiju` | `fastq.screen_taxonomy` | `science/docs/upstream/fastq/tools/kaiju/repo` | https://github.com/bioinformatics-centre/kaiju |
| `source.fastq.tool.kraken2.upstream` | `kraken2` | `fastq.screen_taxonomy` | `science/docs/upstream/fastq/tools/kraken2/repo` | https://github.com/DerrickWood/kraken2 |
| `source.fastq.tool.krakenuniq.upstream` | `krakenuniq` | `fastq.screen_taxonomy` | `science/docs/upstream/fastq/tools/krakenuniq/repo` | https://github.com/fbreitwieser/krakenuniq |
| `source.fastq.tool.leehom.upstream` | `leehom` | `fastq.merge_pairs,fastq.trim_reads` | `science/docs/upstream/fastq/tools/leehom/repo` | https://github.com/grenaud/leeHom |
| `source.fastq.tool.lighter.upstream` | `lighter` | `fastq.correct_errors` | `science/docs/upstream/fastq/tools/lighter/repo` | https://github.com/mourisl/Lighter |
| `source.fastq.tool.multiqc.upstream` | `multiqc` | `fastq.report_qc` | `science/docs/upstream/fastq/tools/multiqc/repo` | https://github.com/multiqc/multiqc |
| `source.fastq.tool.prinseq.upstream` | `prinseq` | `fastq.filter_low_complexity,fastq.filter_reads,fastq.profile_read_lengths,fastq.trim_reads` | `science/docs/upstream/fastq/tools/prinseq/repo` | https://github.com/Adrian-Cantu/PRINSEQ-plus-plus |
| `source.fastq.tool.rcorrector.upstream` | `rcorrector` | `fastq.correct_errors` | `science/docs/upstream/fastq/tools/rcorrector/repo` | https://github.com/mourisl/Rcorrector |
| `source.fastq.tool.seqfu.upstream` | `seqfu` | `fastq.normalize_abundance,fastq.profile_read_lengths,fastq.profile_reads` | `science/docs/upstream/fastq/tools/seqfu/repo` | https://github.com/telatin/seqfu2 |
| `source.fastq.tool.seqkit.upstream` | `seqkit` | `fastq.filter_reads,fastq.normalize_abundance,fastq.profile_overrepresented_sequences,fastq.profile_reads,fastq.trim_reads,fastq.trim_terminal_damage` | `science/docs/upstream/fastq/tools/seqkit/repo` | https://github.com/shenwei356/seqkit |
| `source.fastq.tool.seqkit_stats.upstream` | `seqkit_stats` | `fastq.profile_read_lengths,fastq.profile_reads` | `science/docs/upstream/fastq/tools/seqkit_stats/repo` | https://github.com/shenwei356/seqkit |
| `source.fastq.tool.seqpurge.upstream` | `seqpurge` | `fastq.trim_reads` | `science/docs/upstream/fastq/tools/seqpurge/repo` | https://github.com/imgag/ngs-bits |
| `source.fastq.tool.seqtk.upstream` | `seqtk` | `fastq.validate_reads` | `science/docs/upstream/fastq/tools/seqtk/repo` | https://github.com/lh3/seqtk |
| `source.fastq.tool.skewer.upstream` | `skewer` | `fastq.trim_reads` | `science/docs/upstream/fastq/tools/skewer/repo` | https://github.com/relipmoc/skewer |
| `source.fastq.tool.sortmerna.upstream` | `sortmerna` | `fastq.deplete_rrna` | `science/docs/upstream/fastq/tools/sortmerna/repo` | https://github.com/biocore/sortmerna |
| `source.fastq.tool.star.upstream` | `star` | `fastq.index_reference` | `science/docs/upstream/fastq/tools/star/repo` | https://github.com/alexdobin/STAR |
| `source.fastq.tool.trim_galore.upstream` | `trim_galore` | `fastq.trim_reads` | `science/docs/upstream/fastq/tools/trim_galore/repo` | https://github.com/FelixKrueger/TrimGalore |
| `source.fastq.tool.umi_tools.upstream` | `umi_tools` | `fastq.extract_umis` | `science/docs/upstream/fastq/tools/umi_tools/repo` | https://github.com/CGATOxford/UMI-tools |
| `source.fastq.tool.vsearch.upstream` | `vsearch` | `fastq.cluster_otus,fastq.merge_pairs,fastq.remove_chimeras` | `science/docs/upstream/fastq/tools/vsearch/repo` | https://github.com/torognes/vsearch |

## Ready for Manual Download

| Source ID | Tool | FASTQ Stages | Archive Path | Upstream |
| --- | --- | --- | --- | --- |
| `source.fastq.tool.bbduk.upstream` | `bbduk` | `fastq.filter_low_complexity,fastq.filter_reads,fastq.trim_polyg_tails,fastq.trim_reads` | `science/docs/upstream/fastq/tools/bbduk/download` | https://bbmap.org/tools/bbduk |
| `source.fastq.tool.bbmerge.upstream` | `bbmerge` | `fastq.merge_pairs` | `science/docs/upstream/fastq/tools/bbmerge/download` | https://bbmap.org/tools/bbmerge |
| `source.fastq.tool.clumpify.upstream` | `clumpify` | `fastq.remove_duplicates` | `science/docs/upstream/fastq/tools/clumpify/download` | https://bbmap.org/tools/clumpify |
| `source.fastq.tool.dustmasker.upstream` | `dustmasker` | `fastq.filter_low_complexity` | `science/docs/upstream/fastq/tools/dustmasker/download` | https://www.ncbi.nlm.nih.gov/IEB/ToolBox/CPP_DOC/lxr/source/src/app/dustmasker/ |
| `source.fastq.tool.fastuniq.upstream` | `fastuniq` | `fastq.remove_duplicates` | `science/docs/upstream/fastq/tools/fastuniq/download` | https://sourceforge.net/projects/fastuniq/ |
| `source.fastq.tool.musket.upstream` | `musket` | `fastq.correct_errors` | `science/docs/upstream/fastq/tools/musket/download` | https://sourceforge.net/projects/musket/ |
| `source.fastq.tool.pear.upstream` | `pear` | `fastq.merge_pairs` | `science/docs/upstream/fastq/tools/pear/download` | https://cme.h-its.org/exelixis/web/software/pear/ |
| `source.fastq.tool.trimmomatic.upstream` | `trimmomatic` | `fastq.trim_reads` | `science/docs/upstream/fastq/tools/trimmomatic/download` | https://www.usadellab.org/cms/?page=trimmomatic |

## Paper Archive Worklist

Use [science/docs/upstream/papers/TODO_DOWNLOAD.md](upstream/papers/TODO_DOWNLOAD.md)
for the paired publication or software-citation packets behind these tool
source archives.

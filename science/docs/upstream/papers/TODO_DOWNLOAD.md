# FASTQ Paper Archive Backlog

This file is the operator-facing worklist for local paper payloads under
`science/docs/upstream/papers/`.

Use [README.md](README.md) for the durable paper-root contract and
[../../generated/current/evidence/README.md](../../generated/current/evidence/README.md)
when this backlog needs the wider generated evidence inventory around the FASTQ
paper matrix.

## Purpose

- reserve durable local roots for FASTQ tool publications and software-citation packets
- separate paper payload handling from upstream repository cloning
- make open-access downloads and licensed-access follow-up explicit

## Local Payload Layout

For each `paper_root`, place material under:

- [<paper-id>/original/](<paper-id>/original/)
- [<paper-id>/notes/](<paper-id>/notes/)

Use the same root for PDFs, supplementary files, homepage snapshots, README
exports, and local notes about access constraints.

## Current Local Archive

PDFs are stored as untracked payloads under each paper root. Software-citation
roots intentionally use homepage or repository snapshots rather than paper PDFs.

| Tool | Paper ID | Local Status | Access | Primary Locator |
| --- | --- | --- | --- | --- |
| `adapterremoval` | `paper.fastq.adapterremoval.schubert-2016` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC4751634/ |
| `alientrimmer` | `paper.fastq.alientrimmer.criscuolo-2013` | PDF archived | `licensed_or_abstract_only` | https://pubmed.ncbi.nlm.nih.gov/23912058/ |
| `alientrimmer` | `paper.fastq.alientrimmer.criscuolo-2014-commentary` | supporting PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC4026695/ |
| `atropos` | `paper.fastq.atropos.didion-2017` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC5581536/ |
| `bayeshammer` | `paper.fastq.bayeshammer.nikolenko-2013` | PDF archived | `open_access_publisher` | https://link.springer.com/article/10.1186/1471-2164-14-S1-S7 |
| `bbmerge` | `paper.fastq.bbmerge.bushnell-2017` | PDF archived | `open_access_publisher` | https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0185056 |
| `bowtie2` | `paper.fastq.bowtie2.langmead-2012` | PDF archived | `open_access_pmc` | https://www.nature.com/articles/nmeth.1923 |
| `bowtie2_build` | `paper.fastq.bowtie2-build.langmead-2012` | PDF archived | `open_access_pmc` | https://www.nature.com/articles/nmeth.1923 |
| `centrifuge` | `paper.fastq.centrifuge.kim-2016` | PDF archived | `open_access_pmc` | https://genome.cshlp.org/content/26/12/1721 |
| `cutadapt` | `paper.fastq.cutadapt.martin-2011` | PDF archived | `open_access_publisher` | https://journal.embnet.org/index.php/embnetjournal/article/view/200 |
| `dada2` | `paper.fastq.dada2.callahan-2016` | PDF archived | `licensed_or_abstract_only` | https://www.nature.com/articles/nmeth.3869 |
| `diamond` | `paper.fastq.diamond.buchfink-2015` | PDF archived | `licensed_or_abstract_only` | https://www.nature.com/articles/nmeth.3176 |
| `dustmasker` | `paper.fastq.dustmasker.morgulis-2006` | PDF archived | `licensed_or_abstract_only` | https://journals.sagepub.com/doi/abs/10.1089/cmb.2006.13.1028 |
| `fastp` | `paper.fastq.fastp.chen-2018` | PDF archived | `open_access_pmc` | https://academic.oup.com/bioinformatics/article/34/17/i884/5093234 |
| `fastq_screen` | `paper.fastq.fastq-screen.wingett-2018` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC6124377/ |
| `fastuniq` | `paper.fastq.fastuniq.xu-2012` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC3527383/ |
| `flash2` | `paper.fastq.flash.magoc-2011` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC3198573/ |
| `fqtools` | `paper.fastq.fqtools.droop-2016` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC4908325/ |
| `kaiju` | `paper.fastq.kaiju.menzel-2016` | PDF archived | `open_access_publisher` | https://www.nature.com/articles/ncomms11257 |
| `kraken2` | `paper.fastq.kraken2.wood-2019` | PDF archived | `open_access_publisher` | https://genomebiology.biomedcentral.com/articles/10.1186/s13059-019-1891-0 |
| `krakenuniq` | `paper.fastq.krakenuniq.breitwieser-2018` | PDF archived | `open_access_publisher` | https://genomebiology.biomedcentral.com/articles/10.1186/s13059-018-1568-0 |
| `leehom` | `paper.fastq.leehom.renaud-2014` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC4191382/ |
| `lighter` | `paper.fastq.lighter.song-2014` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC4248469/ |
| `multiqc` | `paper.fastq.multiqc.ewels-2016` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC5039924/ |
| `musket` | `paper.fastq.musket.liu-2013` | PDF archived | `open_access_publisher` | https://academic.oup.com/bioinformatics/article/29/3/308/257257 |
| `pear` | `paper.fastq.pear.zhang-2014` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC3933873/ |
| `prinseq` | `paper.fastq.prinseq.schmieder-2011` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC3051327/ |
| `rcorrector` | `paper.fastq.rcorrector.song-2015` | PDF archived | `open_access_publisher` | https://gigascience.biomedcentral.com/articles/10.1186/s13742-015-0089-y |
| `seqfu` | `paper.fastq.seqfu.telatin-2021` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC8148589/ |
| `seqkit` | `paper.fastq.seqkit.shen-2016` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC5051824/ |
| `seqkit_stats` | `paper.fastq.seqkit-stats.shen-2016` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC5051824/ |
| `seqpurge` | `paper.fastq.seqpurge.stenzel-2016` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC4862148/ |
| `skewer` | `paper.fastq.skewer.jiang-2014` | PDF archived | `open_access_publisher` | https://bmcbioinformatics.biomedcentral.com/articles/10.1186/1471-2105-15-182 |
| `sortmerna` | `paper.fastq.sortmerna.kopylova-2012` | PDF archived | `open_access_publisher` | https://academic.oup.com/bioinformatics/article/28/24/3211/246053 |
| `trimmomatic` | `paper.fastq.trimmomatic.bolger-2014` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC4103590/ |
| `umi_tools` | `paper.fastq.umi-tools.smith-2017` | PDF archived | `open_access_publisher` | https://genome.cshlp.org/content/27/3/491 |
| `vsearch` | `paper.fastq.vsearch.rognes-2016` | PDF archived | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC5075697/ |

## Software-Citation Roots

| Tool | Paper ID | Local Status | Primary Locator |
| --- | --- | --- | --- |
| `bbduk` | `paper.fastq.bbduk.bbtools-software-citation` | homepage/repository snapshot archived | https://bbmap.org/tools/bbduk |
| `bijux_dna` | `paper.fastq.bijux-dna.software-citation` | repository snapshot archived | https://github.com/bijux/bijux-genomics |
| `clumpify` | `paper.fastq.clumpify.bbtools-software-citation` | homepage/repository snapshot archived | https://bbmap.org/tools/clumpify |
| `fastq_scan` | `paper.fastq.fastq-scan.software-citation` | homepage/repository snapshot archived | https://github.com/rpetit3/fastq-scan |
| `fastqc` | `paper.fastq.fastqc.software-citation` | homepage/repository snapshot archived | https://www.bioinformatics.babraham.ac.uk/projects/fastqc/ |
| `fastqvalidator` | `paper.fastq.fastqvalidator.software-citation` | homepage/repository snapshot archived | https://github.com/statgen/fastQValidator |
| `fastx_clipper` | `paper.fastq.fastx-toolkit.software-citation` | homepage/repository snapshot archived | https://github.com/agordon/fastx_toolkit |
| `seqtk` | `paper.fastq.seqtk.software-citation` | homepage/repository snapshot archived | https://github.com/lh3/seqtk |
| `trim_galore` | `paper.fastq.trim-galore.software-citation` | homepage/repository snapshot archived | https://doi.org/10.5281/zenodo.7598955 |

## PDF Follow-Up

There are no remaining mapped FASTQ paper roots waiting on a local payload at
this time. Keep this section empty until a compiled closure report shows a new
paper root with `archive_status = missing`.

## Notes

- [TOOL_PAPER_MAP.tsv](TOOL_PAPER_MAP.tsv) is the tracked machine-readable map.
- [science/generated/current/evidence/fastq_paper_archive_matrix.tsv](../../generated/current/evidence/fastq_paper_archive_matrix.tsv)
  is the compiled review surface for current FASTQ paper roots and archive
  status.
- Trim Galore, seqtk, fastx_clipper, and fastqvalidator are treated as
  software-citation roots because the governed evidence does not identify a
  dedicated peer-reviewed methods paper for those tool surfaces.
- Seqtk keeps the FastQDesign Nature paper as supporting citation context for
  repository-based software references, not as a seqtk methods paper.
- fastqvalidator keeps the VirIdAl pipeline paper as supporting citation
  context because it cites the fastQValidator GitHub repository.
- Formal papers should prefer the publisher DOI landing page as the primary
  locator, with PubMed/PMC retained as supporting index and access locators.
  Software-only roots should prefer durable software-release DOI records when
  available, then maintained repository/project pages.

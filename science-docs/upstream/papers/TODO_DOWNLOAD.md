# FASTQ Paper Archive Backlog

This file is the operator-facing worklist for local paper payloads under
`science-docs/upstream/papers/`.

## Purpose

- reserve durable local roots for FASTQ tool publications and software-citation packets
- separate paper payload handling from upstream repository cloning
- make open-access downloads and licensed-access follow-up explicit

## Local Payload Layout

For each `paper_root`, place material under:

- `science-docs/upstream/papers/<paper-id>/original/`
- `science-docs/upstream/papers/<paper-id>/notes/`

Use the same root for PDFs, supplementary files, homepage snapshots, README
exports, and local notes about access constraints.

## Ready for Local Archive

| Tool | Paper Root | Paper Status | Access | Primary Locator | What To Save |
| --- | --- | --- | --- | --- | --- |
| `atropos` | `science-docs/upstream/papers/paper.fastq.atropos.didion-2017` | `mapped` | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC5581536/ | PMC HTML/PDF and any supplementary notes needed for trimming-method claims |
| `cutadapt` | `science-docs/upstream/papers/paper.fastq.cutadapt.martin-2011` | `mapped` | `open_access_publisher` | https://journal.embnet.org/index.php/embnetjournal/article/view/200 | publisher HTML/PDF, citation metadata, and a local note that the upstream repo/docs stay paired with this paper root |
| `dada2` | `science-docs/upstream/papers/paper.fastq.dada2.callahan-2016` | `mapped` | `licensed_or_abstract_only` | https://www.nature.com/articles/nmeth.3869 | publisher PDF when licensed access is available, plus local notes about access |
| `diamond` | `science-docs/upstream/papers/paper.fastq.diamond.buchfink-2015` | `mapped` | `licensed_or_abstract_only` | https://pubmed.ncbi.nlm.nih.gov/25402007/ | publisher PDF when licensed access is available, plus abstract metadata |
| `dustmasker` | `science-docs/upstream/papers/paper.fastq.dustmasker.selection-pending` | `mapped` | `licensed_or_abstract_only` | https://pubmed.ncbi.nlm.nih.gov/16796549/ | paper PDF when available and local note that the NCBI dustmasker page points to this citation |
| `fastp` | `science-docs/upstream/papers/paper.fastq.fastp.chen-2018` | `mapped` | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC6129281/ | PMC HTML/PDF and citation metadata |
| `fastqvalidator` | `science-docs/upstream/papers/paper.fastq.fastqvalidator.software-citation` | `software_citation_only` | `project_repository` | https://github.com/statgen/fastQValidator | repository snapshot, release metadata, and local notes explaining that no canonical methods paper is currently governed |
| `fastqc` | `science-docs/upstream/papers/paper.fastq.fastqc.selection-pending` | `software_citation_only` | `project_homepage` | https://www.bioinformatics.babraham.ac.uk/projects/fastqc/ | homepage snapshot, release notes, and README export instead of a paper PDF |
| `fastuniq` | `science-docs/upstream/papers/paper.fastq.fastuniq.xu-2012` | `mapped` | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC3527383/ | PMC HTML/PDF and citation metadata |
| `seqfu` | `science-docs/upstream/papers/paper.fastq.seqfu.telatin-2021` | `mapped` | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC8148589/ | PMC HTML/PDF and citation metadata |
| `seqkit` | `science-docs/upstream/papers/paper.fastq.seqkit.shen-2016` | `mapped` | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC5051824/ | PMC HTML/PDF and citation metadata |
| `seqkit_stats` | `science-docs/upstream/papers/paper.fastq.seqkit-stats.shen-2016` | `mapped` | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC5051824/ | same publication packet as `seqkit`, stored under the stats-specific root |
| `seqpurge` | `science-docs/upstream/papers/paper.fastq.seqpurge.stenzel-2016` | `mapped` | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC4862148/ | PMC HTML/PDF and citation metadata |
| `umi_tools` | `science-docs/upstream/papers/paper.fastq.umi-tools.smith-2017` | `mapped` | `open_access_publisher` | https://genome.cshlp.org/content/27/3/491 | publisher HTML/PDF, supplemental files when useful, and citation metadata |
| `vsearch` | `science-docs/upstream/papers/paper.fastq.vsearch.rognes-2016` | `mapped` | `open_access_pmc` | https://pmc.ncbi.nlm.nih.gov/articles/PMC5075697/ | PMC HTML/PDF and citation metadata |

## Notes

- `TOOL_PAPER_MAP.tsv` is the tracked machine-readable map.
- `science/generated/current/evidence/fastq_paper_archive_matrix.tsv` is the
  compiled review surface for current FASTQ paper roots and archive status.

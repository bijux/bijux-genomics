# FASTQ References

## What
Primary references supporting FASTQ-stage tools, metrics, and domain decisions.

## Why
The FASTQ domain is only reviewable when each supported backend can be traced to a peer-reviewed method paper or, when no dedicated paper is available, to a stable software citation.

## Non-goals
- Exhaustive literature review.
- Replacement for upstream manuals.
- Claiming a paper for software that is only documented as software.

## Contracts
- Every admitted FASTQ backend must have either a paper locator or an explicit software-citation locator.
- Evidence gaps must remain visible in the generated evidence backlog rather than being hidden in prose.

## Citation Policy
- Prefer DOI-bearing publisher pages or stable full-text records for papers.
- Use software/project pages for tools without a dedicated peer-reviewed paper.
- Use Zenodo DOI records for archived software releases when the tool has no dedicated paper and the release DOI is the most stable citation.
- Use PubMed or PMC mirrors as access copies, not as the canonical locator, when a DOI-bearing paper page is available.
- Keep generated closure state in
  [science/generated/current/evidence/README.md](../../../science/generated/current/evidence/README.md);
  this file remains the human-readable reference guide.

## Quality Control and Profiling
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| fastqc | `fastq.validate_reads`, `fastq.detect_adapters`, `fastq.profile_overrepresented_sequences` | software citation; no dedicated journal paper | https://www.bioinformatics.babraham.ac.uk/projects/fastqc/ |
| fastq-scan | `fastq.validate_reads`, `fastq.profile_overrepresented_sequences` | software citation; no dedicated journal paper confirmed | https://github.com/rpetit3/fastq-scan |
| fqtools | `fastq.validate_reads` | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC4908325/ |
| seqtk | `fastq.validate_reads` | software citation | https://github.com/lh3/seqtk |

## Trimming, Filtering, and Complexity
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| fastp | `fastq.trim_reads`, `fastq.filter_reads`, `fastq.trim_polyg_tails`, `fastq.profile_read_lengths` | paper | https://academic.oup.com/bioinformatics/article/34/17/i884/5093234 |
| cutadapt | `fastq.trim_reads`, `fastq.trim_terminal_damage`, `fastq.normalize_primers` | paper | https://journal.embnet.org/index.php/embnetjournal/article/view/200 |
| atropos | `fastq.trim_reads` | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC5581536/ |
| adapterremoval | `fastq.trim_reads` | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC4751634/ |
| trimmomatic | `fastq.trim_reads` | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC4103590/ |
| trim_galore | `fastq.trim_reads` | archived software release DOI; no dedicated journal paper confirmed | https://doi.org/10.5281/zenodo.7598955 |
| bbduk | `fastq.trim_reads`, `fastq.filter_reads`, `fastq.trim_polyg_tails`, `fastq.filter_low_complexity` | BBTools software citation; no dedicated BBDuk paper confirmed | https://archive.jgi.doe.gov/data-and-tools/software-tools/bbtools/bb-tools-user-guide/bbduk-guide/ |
| prinseq | `fastq.filter_reads`, `fastq.filter_low_complexity`, read-length profiling | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC3051327/ |
| seqkit | `fastq.filter_reads`, `fastq.profile_overrepresented_sequences`, `fastq.trim_terminal_damage`, `fastq.normalize_abundance` | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC5051824/ |
| seqfu | planned read-length and abundance support | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC8148589/ |
| dustmasker | `fastq.filter_low_complexity` planned support | paper for symmetric DUST implementation | https://journals.sagepub.com/doi/abs/10.1089/cmb.2006.13.1028 |

## Merging and Duplicate Removal
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| pear | `fastq.merge_pairs` | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC3933873/ |
| vsearch | `fastq.merge_pairs`, `fastq.remove_chimeras`, `fastq.cluster_otus` | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC5075697/ |
| bbmerge | `fastq.merge_pairs` | BBTools-associated paper | https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0185056 |
| flash2 | `fastq.merge_pairs` | paper for FLASH method family | https://pmc.ncbi.nlm.nih.gov/articles/PMC3198573/ |
| leehom | `fastq.merge_pairs` | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC4191382/ |
| fastuniq | `fastq.remove_duplicates` | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC3527383/ |
| clumpify | `fastq.remove_duplicates` | BBTools software citation; no dedicated Clumpify paper confirmed | https://archive.jgi.doe.gov/data-and-tools/software-tools/bbtools/bb-tools-user-guide/clumpify-guide/ |

## Depletion, Indexing, and Alignment
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| bowtie2 | `fastq.deplete_host`, `fastq.deplete_reference_contaminants` | paper | https://www.nature.com/articles/nmeth.1923 |
| bowtie2-build | `fastq.index_reference` | paper via Bowtie 2 | https://www.nature.com/articles/nmeth.1923 |
| sortmerna | `fastq.deplete_rrna` | paper | https://academic.oup.com/bioinformatics/article/28/24/3211/246053 |
| star | planned indexing and host-depletion support | paper | https://academic.oup.com/bioinformatics/article/29/1/15/272537 |

## Taxonomic Screening
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| kraken2 | `fastq.screen_taxonomy` | paper | https://genomebiology.biomedcentral.com/articles/10.1186/s13059-019-1891-0 |
| krakenuniq | `fastq.screen_taxonomy` | paper | https://genomebiology.biomedcentral.com/articles/10.1186/s13059-018-1568-0 |
| centrifuge | `fastq.screen_taxonomy` | paper | https://genome.cshlp.org/content/26/12/1721 |
| kaiju | `fastq.screen_taxonomy` | paper | https://www.nature.com/articles/ncomms11257 |
| diamond | planned protein-search screening support | paper | https://www.nature.com/articles/nmeth.3176 |

## Amplicon, Error Correction, and UMI Handling
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| dada2 | `fastq.infer_asvs` | paper | https://www.nature.com/articles/nmeth.3869 |
| rcorrector | `fastq.correct_errors` | paper | https://gigascience.biomedcentral.com/articles/10.1186/s13742-015-0089-y |
| musket | `fastq.correct_errors` | paper | https://academic.oup.com/bioinformatics/article/29/3/308/257257 |
| lighter | `fastq.correct_errors` | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC4248469/ |
| bayeshammer | `fastq.correct_errors` | paper | https://link.springer.com/article/10.1186/1471-2164-14-S1-S7 |
| umi_tools | `fastq.extract_umis` | paper | https://genome.cshlp.org/content/27/3/491 |

## Report Aggregation
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| multiqc | `fastq.report_qc` | paper | https://pmc.ncbi.nlm.nih.gov/articles/PMC5039924/ |

## Evidence Gaps
- Software-only tools remain valid runtime backends only when their stage contracts are otherwise governed and their software citation is explicit.
- Missing local paper/archive payloads are tracked in
  [science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv](../../../science/generated/current/evidence/fastq_missing_closure_prerequisites.tsv),
  [science/generated/current/evidence/fastq_paper_archive_matrix.tsv](../../../science/generated/current/evidence/fastq_paper_archive_matrix.tsv),
  and [science/generated/current/evidence/fastq_download_backlog.tsv](../../../science/generated/current/evidence/fastq_download_backlog.tsv),
  plus `/Users/bijan/bijux/NEEDED.md`.
- Unsupported tools must not be listed here as admitted backends unless they are present in
  [domain/fastq/execution_support.yaml](../../../domain/fastq/execution_support.yaml)
  and the corresponding stage manifest.

# VCF References

## What
Governed reference and citation ledger for VCF-stage tools and method families.

## Why
The VCF domain spans supported calling and downstream analysis families at different citation and execution-evidence closure states. This file makes tool-to-stage applicability explicit while keeping incomplete citation closure visible instead of hidden behind prose.

## Non-goals
- Claiming complete citation closure or execution evidence for every supported downstream tool.
- Replacing tool manifests or generated evidence reports.

## Contracts
- Every VCF tool in `domain/vcf/tools/*.yaml` must appear here exactly once.
- `Applies to` must mirror the tool manifest `stage_ids`.
- Conservative release-review blocker status for VCF tools now lives in
  [../../../science/docs/upstream/vcf/VCF_DOWNSTREAM_CLOSURE_LEDGER.tsv](https://github.com/bijux/bijux-genomics/blob/main/science/docs/upstream/vcf/VCF_DOWNSTREAM_CLOSURE_LEDGER.tsv).
- Incomplete citations must retain an honest `planned` status rather than fake-closed claims.

## Supported Calling and Filtering
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| bcftools | `vcf.call`, `vcf.call_gl`, `vcf.call_diploid`, `vcf.call_pseudohaploid`, `vcf.damage_filter`, `vcf.gl_propagation`, `vcf.filter`, `vcf.qc`, `vcf.stats` | DOI captured in governed tool contract | https://github.com/samtools/bcftools |
| angsd | `vcf.call_gl`, `vcf.call_pseudohaploid` | paper-style citation captured in governed tool contract; execution requires an admitted operator-provided artifact | https://github.com/ANGSD/angsd |

## Downstream Inference Families
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| beagle | `vcf.imputation_metrics`, `vcf.impute`, `vcf.phasing` | governed paper locator present | https://faculty.washington.edu/browning/beagle/beagle.html |
| eagle | `vcf.phasing` | governed method locator present | https://www.nature.com/articles/ng.3679 |
| eigensoft | `vcf.pca`, `vcf.population_structure` | planned citation closure | https://github.com/DReichLab/EIG |
| germline | `vcf.ibd` | governed paper locator present; promotion and runtime evidence still pending | https://github.com/gusevlab/germline/tree/f8029fcf469a09a83aa1e0d1fb45769f6dcb3c6d |
| glimpse | `vcf.imputation_metrics`, `vcf.impute` | governed upstream source locator present | https://github.com/odelaneau/GLIMPSE |
| ibdhap | `vcf.ibd` | MORGAN 3.4 release and IBDHap user manual pinned in the governed tool contract | https://sites.stat.washington.edu/thompson/Genepi/MORGAN/Morgan.shtml |
| ibdne | `vcf.demography` | planned citation closure | https://faculty.washington.edu/browning/ibdne.shtml |
| impute5 | `vcf.imputation_metrics`, `vcf.impute` | Rubinacci, Delaneau, and Marchini (2020), DOI `10.1371/journal.pgen.1009049`; external academic-use license required | https://jmarchini.org/software/#impute-5 |
| minimac4 | `vcf.imputation_metrics`, `vcf.impute` | governed upstream source locator present | https://github.com/statgen/Minimac4 |
| plink | `vcf.qc`, `vcf.admixture`, `vcf.population_structure` | governed upstream locator present | https://www.cog-genomics.org/plink/ |
| plink2 | `vcf.qc`, `vcf.pca`, `vcf.admixture`, `vcf.population_structure`, `vcf.roh` | planned citation closure | https://www.cog-genomics.org/plink/2.0/ |
| shapeit | `vcf.phasing` | governed SHAPEIT4 method locator present | https://odelaneau.github.io/shapeit4/ |
| shapeit5 | `vcf.phasing` | governed method locator present | https://www.nature.com/articles/s41588-023-01415-w |

## Failure modes
- A stage listed against the wrong tool creates fake scientific support for a workflow we do not actually admit.
- A tool's paper, release, license, runtime, and scientific qualification are separate evidence dimensions; a resolved upstream locator alone is not a production claim.

# VCF References

## What
Governed reference and citation ledger for VCF-stage tools and method families.

## Why
The VCF domain mixes currently supported calling stages with many planned downstream analysis families. This file makes tool-to-stage applicability explicit while keeping incomplete citation closure visible instead of hidden behind prose.

## Non-goals
- Claiming complete citation closure for every planned downstream tool.
- Replacing tool manifests or generated evidence reports.

## Contracts
- Every VCF tool in `domain/vcf/tools/*.yaml` must appear here exactly once.
- `Applies to` must mirror the tool manifest `stage_ids`.
- Conservative release-review blocker status for VCF tools now lives in
  [../../../science/docs/upstream/vcf/VCF_DOWNSTREAM_CLOSURE_LEDGER.tsv](../../../science/docs/upstream/vcf/VCF_DOWNSTREAM_CLOSURE_LEDGER.tsv).
- Planned tools may use honest `planned` citation status rather than fake-closed claims.

## Supported Calling and Filtering
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| bcftools | `vcf.call`, `vcf.call_gl`, `vcf.call_diploid`, `vcf.call_pseudohaploid`, `vcf.damage_filter`, `vcf.gl_propagation`, `vcf.filter`, `vcf.stats` | DOI captured in governed tool contract | https://github.com/samtools/bcftools |
| angsd | `vcf.call_gl`, `vcf.call_pseudohaploid`, `vcf.damage_filter`, `vcf.gl_propagation` | paper-style citation captured in governed tool contract; runtime promotion still planned | https://github.com/ANGSD/angsd |

## Planned Downstream Inference Families
| Tool | Applies to | Reference status | Primary locator |
| --- | --- | --- | --- |
| beagle | `vcf.phasing`, `vcf.imputation`, `vcf.impute` | planned citation closure | https://faculty.washington.edu/browning/beagle/beagle.html |
| eagle | `vcf.phasing` | planned citation closure | https://alkesgroup.broadinstitute.org/Eagle/ |
| eigensoft | `vcf.pca`, `vcf.population_structure` | planned citation closure | https://github.com/DReichLab/EIG |
| germline | `vcf.ibd` | governed paper locator present; promotion and runtime evidence still pending | https://www.cs.columbia.edu/~gusev/germline/ |
| glimpse | `vcf.impute` | planned citation closure | https://odelaneau.github.io/GLIMPSE/ |
| ibdhap | `vcf.ibd` | placeholder upstream locator still unresolved in governed tool contract | https://example.invalid/ibdhap |
| ibdne | `vcf.demography` | planned citation closure | https://faculty.washington.edu/browning/ibdne.shtml |
| impute5 | `vcf.impute` | planned citation closure | https://jmarchini.org/software/#impute-5 |
| minimac4 | `vcf.impute` | planned citation closure | https://genome.sph.umich.edu/wiki/Minimac4 |
| plink | `vcf.qc`, `vcf.admixture`, `vcf.population_structure` | planned citation closure | https://www.cog-genomics.org/plink/ |
| plink2 | `vcf.qc`, `vcf.pca`, `vcf.admixture`, `vcf.population_structure`, `vcf.roh` | planned citation closure | https://www.cog-genomics.org/plink/2.0/ |
| shapeit5 | `vcf.phasing` | planned citation closure | https://odelaneau.github.io/shapeit5/ |

## Failure modes
- A stage listed against the wrong tool creates fake scientific support for a workflow we do not actually admit.
- Placeholder upstream locators such as `https://example.invalid/ibdhap` must stay visible until the governed tool contract is repaired.

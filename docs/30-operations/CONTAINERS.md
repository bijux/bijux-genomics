# Containers

## What

Defines container identity, build, execution, and evidence rules.

## Why

An image digest makes the runtime repeatable, but it does not prove that a
tool handles real data correctly. Bijux Genomics therefore records source,
build, runtime, stage, scientific, benchmark, security, and release evidence
as separate claims.

## Purpose

Provide one operational contract for local checks, immutable image admission,
LUNARC Slurm execution, and public evidence reporting.

## Scope

Applies to catalog entries, Docker and Apptainer recipes, source locks, site
configuration, run manifests, and reviewed evidence summaries.

## Non-goals

- Managing registries for the user.
- Treating a version or help command as scientific validation.
- Publishing raw corpora, large run products, or credential values.

## Contracts

- Production runs use immutable source, image, repository, and input
  identities.
- Apptainer is the governed LUNARC runtime. Docker support is declared per
  tool and architecture; absence is explicit rather than simulated.
- Version, help, negative, and real-data commands are distinct checks.
- Raw results stay in the configured study tree. Small reviewed reports carry
  immutable evidence and accounting checksums into the repository.
- Unsupported tools remain visible with a durable reason.

## Configured HPC execution

Reusable code contains no LUNARC study path. The governed YAML site contract
defines the study, repository, frozen-checkout, runtime, account, and partition
settings. Every run archives that contract with its scientific pipeline and a
credential-reference document that contains no secrets.

The executor validates configured roots, exact checksums, a clean frozen Git
commit, output uniqueness, and Slurm policy before submission. It preserves
failed runs and collects terminal accounting separately.

## Real evidence

The [qualification matrix](container-evidence/QUALIFICATION.md) is the current
cross-tool truth surface. It derives readiness dimension by dimension and keeps
missing scientific, benchmark, security, or release evidence visible. The
[image inventory](container-evidence/IMAGE_INVENTORY.md) and
[runtime verification report](container-evidence/RUNTIME_VERIFICATION.md)
record the admitted SIFs and their version/help checks without overstating
those checks as real-data or scientific validation.

The [LUNARC FASTQ-to-BAM report](container-evidence/FASTQ_BAM.md)
records a real horse WGS trimming, alignment, sorting, indexing, and QC chain.
The [paired-read trimming report](container-evidence/PAIRED_READ_TRIMMING.md)
compares two governed trimming executions while stating why the single runs do
not yet form a fair benchmark.
The [LUNARC ancient-DNA damage report](container-evidence/ANCIENT_DAMAGE.md)
records ngsBriggs input-contract correction and real Slurm execution on a
public ancient-horse alignment.
The [ancient-DNA genotype-likelihood report](container-evidence/ANCIENT_GENOTYPE_LIKELIHOODS.md)
records the governed ANGSD execution and its interpretation boundary.
The [LUNARC reference-complexity report](container-evidence/REFERENCE_COMPLEXITY.md)
records real EquCab3.0 interval extraction and DustMasker execution.
The [LUNARC amplicon ASV report](container-evidence/AMPLICON_ASV.md) records
single-end and overlapping paired-end DADA2 inference, including the preserved
runtime-dependency and incompatible-corpus failures that shaped the final
contract.
The [LUNARC VCF phasing report](container-evidence/VCF_PHASING.md)
records a real 2,504-sample BCFtools, SHAPEIT4, SHAPEIT5, and Eagle execution
chain. The
[LUNARC VCF relatedness report](container-evidence/VCF_RELATEDNESS.md)
records passing chromosome-wide IBDseq and IBDNe stage execution while
preserving the population and multi-chromosome requirements for scientific
interpretation. The reports explain why stage success is not yet a scientific
accuracy or release claim.

## Failure modes

- Missing digest blocks promotion to production.
- A path outside the configured root blocks submission.
- An existing result directory blocks overwrite and requires a new run ID.
- Missing input normalization is recorded as a failed run and corrected by an
  explicit governed derivation.

## Canonical admission reference

- Use [docs/50-reference/TOOL_ADMISSION.md](../50-reference/TOOL_ADMISSION.md)
  as the single authoritative "how to add tool" checklist.

## References

- [Container implementation](https://github.com/bijux/bijux-genomics/tree/main/containers)
- [Container contract documentation](https://github.com/bijux/bijux-genomics/tree/main/containers/docs)
- [Release checklist](https://github.com/bijux/bijux-genomics/blob/main/containers/docs/RELEASE_CHECKLIST.md)
- [LUNARC execution contract](https://github.com/bijux/bijux-genomics/blob/main/containers/docs/LUNARC_EXECUTION_CONTRACT.md)
- [Evidence report index](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/README.md)

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

The [LUNARC FASTQ-to-BAM report](container-evidence/FASTQ_BAM.md)
records a real horse WGS trimming, alignment, sorting, indexing, and QC chain.
The [LUNARC VCF phasing report](container-evidence/VCF_PHASING.md)
records a real 2,504-sample BCFtools, SHAPEIT4, SHAPEIT5, and Eagle execution
chain. The
[LUNARC VCF relatedness report](container-evidence/VCF_RELATEDNESS.md)
records a passing IBDseq run and the incompatible-input boundary that prevents
an honest IBDNe claim. The reports explain why stage success is not yet a
scientific accuracy or release claim.

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

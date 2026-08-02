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

The [LUNARC FASTQ-to-BAM report](../../containers/reports/lunarc-fastq-bam.md)
records a real horse WGS trimming, alignment, sorting, indexing, and QC chain.
The [LUNARC VCF phasing report](../../containers/reports/lunarc-vcf-phasing.md)
records a real 2,504-sample BCFtools, SHAPEIT5, and Eagle execution chain. Both
reports explain why stage success is not yet a scientific accuracy or release
claim.

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
- [containers/index.md](../../containers/index.md)
- [containers/docs/index.md](../../containers/docs/index.md)
- [containers/README.md](../../containers/README.md)
- [containers/docs/RELEASE_CHECKLIST.md](../../containers/docs/RELEASE_CHECKLIST.md)
- [containers/docs/LUNARC_EXECUTION_CONTRACT.md](../../containers/docs/LUNARC_EXECUTION_CONTRACT.md)
- [containers/reports/README.md](../../containers/reports/README.md)

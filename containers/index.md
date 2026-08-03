# Containers

Bijux Genomics packages third-party genomics tools for reproducible local and
LUNARC execution. A recipe in this repository is only source code. It becomes
an admitted image after its build provenance and SIF checksum are recorded; it
becomes runtime-verified after its real version and help entry points pass; and
it gains real-data evidence only through an archived Slurm run. None of those
steps alone establishes scientific validity or release readiness.

## Evidence available now

- [Current qualification](reports/qualification.md): the published inventory
  supports 85 current runtime checks, 46 real-data checks, and one governed
  benchmark. Only six images presently have build provenance whose recipe and
  repository-owned build inputs can be revalidated against this commit. No
  tool yet has scientific, security, or release acceptance.
- [Canonical image storage](reports/image-storage.md): a previous governed
  campaign moved 14 unreferenced obsolete SIFs into a checksummed, reversible
  quarantine. Its counts describe that campaign commit, not the current tree.
- [Complete-roster rebuild](reports/lunarc-image-rebuild.md): a previous frozen
  commit rebuilt 87 then-retained Apptainer images. This remains historical
  build evidence while the current 85-image roster is rebuilt and published.
- [Image inventory](reports/image-inventory.md) and [runtime
  verification](reports/runtime-verification.md): all 85 supported tools have
  admitted images and passing version/help records. The inventory predates
  complete publication of repository-owned build-input identities, so those
  historical records do not establish current recipe freshness on their own.
- [Real executions](reports/run-evidence.md): successful, failed, and incomplete
  LUNARC attempts are all retained; scientific assessment remains separate.
- [Scientific reports](reports/README.md): reviewed cross-tool reports state the
  corpus, acceptance boundary, evidence, and remaining limitations.

## Reproduce or review a run

Start with [the container operating guide](README.md). It gives the exact image
inventory, build, quarantine, runtime-verification, Slurm submission,
accounting, and evidence-import commands. Every current Slurm specification has
two public documents:

- `config-pipeline.yml` freezes repository, image, input, command, resource,
  output, and acceptance facts;
- `config-credential.yml` contains credential references and redaction rules,
  never credential values.

Completed attempts live under [`containers/runs/`](runs/README.md). Large SIFs,
inputs, logs, and results stay in the governed LUNARC study tree; tracked locks
record their identities and checksums.

## Authorities

- [Container catalog](contracts/catalog.toml): identity, ownership, support,
  source, runtime, stage, and evidence requirements.
- [Version lock](versions/lock.json): generated immutable version/source facts.
- [LUNARC execution contract](docs/LUNARC_EXECUTION_CONTRACT.md): governed paths,
  scratch, submission, collection, and recovery behavior.
- [Evidence boundaries](docs/SCIENCE_EVIDENCE_BOUNDARY.md): what runtime, stage,
  science, benchmark, security, and release claims mean.
- [Evidence qualification](docs/EVIDENCE_QUALIFICATION.md): how catalog support
  and independently derived production evidence remain separate.
- [Detailed documentation index](docs/index.md): policies and operator reference.

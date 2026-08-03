# Containers

Bijux Genomics packages third-party genomics tools for reproducible local and
LUNARC execution. A recipe in this repository is only source code. It becomes
an admitted image after its build provenance and SIF checksum are recorded; it
becomes runtime-verified after its real version and help entry points pass; and
it gains real-data evidence only through an archived Slurm run. None of those
steps alone establishes scientific validity or release readiness.

## What is verified now

- [Image inventory](reports/image-inventory.md): every supported Apptainer
  recipe has a current, immutable amd64 SIF. Unsupported tools remain explicit.
- [Runtime verification](reports/runtime-verification.md): version and help
  commands are checked inside every admitted image with network access disabled.
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

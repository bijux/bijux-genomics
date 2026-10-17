# bijux-dna-domain-fastq Effects

Pure domain queries and value kernels perform no filesystem publication. Retained
path-based library operations also acquire and publish caller-selected files;
library-only ownership does not make those operations read-only.

## Allowed Effects

- Read governed domain, bank, reference, fixture, and snapshot inputs.
- Parse JSON, YAML, TOML-like manifests, FASTQ headers, gzip-compressed FASTQ discovery inputs, and
  governed tool report text.
- Hash governed contract content for deterministic provenance.
- Emit tracing diagnostics.
- `stages::contract` path operations may read FASTQ/table/report inputs, create
  output directories and write transformed FASTQ, FASTA, table, report or manifest
  files. `normalize_outputs` can rename files to canonical output names.
- The `run` facade reexports core `write_input_assessment`, which publishes the
  caller-selected assessment file. This does not add a CLI or execute a provider.
- Write snapshots only through test commands that intentionally update snapshot artifacts.

## Forbidden Effects

- Process spawning or shelling out.
- Network access.
- Container execution.
- Pipeline scheduling or runtime state mutation.
- Generated config writes.
- Planner or runner selection side effects.

## Determinism

Catalog iteration, bank hashes, contract JSON, snapshot output, and fixture discovery must be stable
for identical input. Filesystem behavior also depends on current paths and existing
outputs; these APIs do not promise atomic snapshots, transactional multi-file
publication or confinement against concurrent filesystem replacement.

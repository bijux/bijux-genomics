# bijux-dna-domain-fastq Effects

The crate is a pure domain library with narrow deterministic file reads for governed assets and
fixtures.

## Allowed Effects

- Read governed domain, bank, reference, fixture, and snapshot inputs.
- Parse JSON, YAML, TOML-like manifests, FASTQ headers, gzip-compressed FASTQ discovery inputs, and
  governed tool report text.
- Hash governed contract content for deterministic provenance.
- Emit tracing diagnostics.
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
for identical input.

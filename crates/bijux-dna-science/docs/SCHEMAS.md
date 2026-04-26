# bijux-dna-science Schemas

## Authored Specs
Authored YAML specs live under `science/specs/**` and declare explicit schema versions for sources,
evidence, claims, assumptions, reasoning, decisions, bindings, and releases.

## Compiled Model
`compile::compile_workspace` loads authored specs, validates cross references, and derives generated
science rows for source inventories, archive gaps, FASTQ environment closure, claim evidence, and
decision reasoning.

## Generated Evidence
Generated TSV and JSON outputs are committed under `science/generated/**` only when they match the
compiled model exactly.

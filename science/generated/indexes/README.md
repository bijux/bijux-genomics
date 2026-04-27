# Generated Science Indexes

`science/generated/indexes/` holds rolled-up generated science entrypoints.

## Role

- expose compact JSON entrypoints derived from the row-level snapshot in
  [../current/README.md](../current/README.md)
- stay downstream of the authored evidence authority in
  [../../specs/evidence/README.md](../../specs/evidence/README.md)
- provide an operator landing point without reopening every emitted TSV

## Governed Output

- [science_index.json](science_index.json) is the top-level rolled-up science
  index for the current committed generated slice

## Adjacent Surface

- [../README.md](../README.md) explains the wider generated-science boundary

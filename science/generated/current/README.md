# Current Science Snapshot

`science/generated/current/` is the committed generated snapshot for the current
workspace state.

## Role

- hold the machine-written evidence rows that correspond to the current authored
  science state
- separate the row-level snapshot from the rolled-up summaries in
  [../indexes/README.md](../indexes/README.md)
- stay downstream of the authored evidence authority in
  [../../specs/evidence/README.md](../../specs/evidence/README.md)

## Governed Subsurface

- [evidence/README.md](evidence/README.md) inventories the committed row-level
  outputs in this snapshot
- [../README.md](../README.md) explains the wider generated-science surface

Refresh this directory through `bijux-dna-science build`; do not hand-maintain
row files here.

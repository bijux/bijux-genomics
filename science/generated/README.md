# Generated Science Outputs

`science/generated/` is the committed compiler output surface for `bijux-genomics`.

## Role

- hold deterministic science outputs emitted by `bijux-dna-science`
- separate compiled evidence from authored records in [science/README.md](../README.md)
- expose stable landing pages for the current snapshot and rolled-up indexes

## Governed Subsurfaces

- [current/README.md](current/README.md) describes the committed current science snapshot
- [indexes/README.md](indexes/README.md) describes the rolled-up JSON entrypoints
- [../specs/evidence/README.md](../specs/evidence/README.md) remains the authored evidence authority that these outputs compile from

Do not hand-edit files under `science/generated/**`. Refresh them through the
`bijux-dna-science build` surface.

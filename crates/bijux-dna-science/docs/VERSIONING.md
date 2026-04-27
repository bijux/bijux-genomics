# bijux-dna-science Versioning

## Release Bundles

Science releases package the compiled evidence state for review without mutating
authored specs or generated current outputs.

The authored release boundary is
[science/specs/releases/README.md](../../../science/specs/releases/README.md),
and the generated current-state boundary is
[science/generated/current/README.md](../../../science/generated/current/README.md).

## Output Root
Release bundles are written below `artifacts/science-releases/<release-id>/`.

## Contents

- Evidence TSV files from
  [science/generated/current/evidence/README.md](../../../science/generated/current/evidence/README.md).
- Science index JSON from
  [science/generated/indexes/README.md](../../../science/generated/indexes/README.md).
- Release metadata derived from the matching manifest.

## Immutability

`release` refuses to overwrite an existing `artifacts/science-releases/<release-id>/`
directory. A changed release must use a new authored release manifest or an explicit
repository-owned removal outside this crate.

## Determinism

Release rendering uses sorted inputs and stable TSV/JSON rendering so the same authored specs produce
the same release bundle.

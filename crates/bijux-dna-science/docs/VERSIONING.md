# bijux-dna-science Versioning

## Release Bundles
Science releases package the compiled evidence state for review without mutating authored specs.

## Output Root
Release bundles are written below `artifacts/science-releases/<release-id>/`.

## Contents
- Evidence TSV files.
- Science index JSON.
- Release metadata derived from the matching manifest.

## Determinism
Release rendering uses sorted inputs and stable TSV/JSON rendering so the same authored specs produce
the same release bundle.

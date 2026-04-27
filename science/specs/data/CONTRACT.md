# Data Specs Contract

This directory is reserved for authored data-plane inputs.

Author only reviewable data-plane specifications here: identifiers, payload
contracts, provenance assumptions, and other inputs that should stay authored
even when downstream compilers generate derived tables.

## Boundaries

- [README.md](README.md) records the current seeded status for this directory.
- [../evidence/README.md](../evidence/README.md) remains the active authored
  authority for the current FASTQ science slice while no dedicated data-plane
  specs are seeded here.
- [../../CONTRACT.md](../../CONTRACT.md) defines the root boundary that keeps
  authored data-plane inputs separate from generated outputs and local archives.
- [../../generated/README.md](../../generated/README.md) is the downstream
  compiled surface that must remain compiler-owned, not hand-maintained here.
- [../../README.md](../../README.md) defines the wider authored, generated, and
  local-archive split for the full science control surface.

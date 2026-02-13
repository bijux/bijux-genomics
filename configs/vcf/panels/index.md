# configs/vcf/panels

## What
Reference panel catalog and lock metadata for downstream VCF workflows.

## Files
- `configs/vcf/panels/panels.toml`
- `configs/vcf/panels/locks/index.md`

## Contracts
- Panel inputs are locked artifacts with pinned version, URL, and checksum.
- No floating URLs or branch-style identifiers are allowed.
- Required metadata for ancestry matching and compatibility:
  - `population_set`
  - `genome_build`
  - `variant_set_compatibility`
  - `license`
  - `citation`

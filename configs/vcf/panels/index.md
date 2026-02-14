# configs/vcf/panels

## What
Reference panel catalog and lock metadata for downstream VCF workflows.

## Files
- `configs/vcf/panels/panels.toml`
- `configs/vcf/panels/locks/index.md`

## Contracts
- Panels are keyed by `{species_id, build_id}` and have locked artifacts per file.
- No floating URLs or branch-style identifiers are allowed.
- Required metadata for ancestry matching and compatibility:
  - `population_set`
  - `species_id`
  - `build_id`
  - `license`
  - `citation`

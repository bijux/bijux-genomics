# Commands

This file is the SSOT for callable operations owned by `bijux-dna-db-ref`.

`bijux-dna-db-ref` does not own a CLI binary. Its managed operations are pure or
read-only library entrypoints.

## Managed Library Operations

| Operation | Rust entrypoint | Purpose |
| --- | --- | --- |
| `resolve-species-alias` | `resolve_species_alias` | Normalize a species alias and choose the requested or default build. |
| `resolve-species-authority` | `resolve_species_authority` | Load the species authority row for a canonical species. |
| `resolve-species-context` | `resolve_species_context` | Project species/build config into a VCF `SpeciesContext`. |
| `resolve-contig-map` | `resolve_contig_map` | Load contig naming and alias metadata for a species/build. |
| `enforce-build-contigs` | `enforce_declared_build_and_contigs` | Validate declared build and observed contigs against authority metadata. |
| `resolve-reference-bank` | `resolve_reference_bank` | Resolve reference bank metadata and checksum contract. |
| `resolve-reference-bundle` | `resolve_reference_bundle` | Resolve a locked reference bundle and normalization policy. |
| `normalize-contig-name` | `normalize_contig_name` | Normalize or reject contig names according to the bundle policy. |
| `reference-provenance` | `reference_provenance` | Build provenance metadata from a resolved bundle. |
| `resolve-genetic-map-bank` | `resolve_genetic_map_bank` | Resolve genetic map bank metadata for a species/build and optional panel. |
| `resolve-organellar-policy` | `resolve_organellar_policy` | Resolve mitochondrial and chloroplast policy metadata. |
| `resolve-default-reference-set` | `resolve_default_reference_set` | Resolve the default reference set for a species and use case. |
| `resolve-sex-chromosome-rule` | `resolve_sex_chromosome_rule` | Resolve sex chromosome and PAR metadata. |
| `resolve-coverage-profile` | `resolve_coverage_profile` | Resolve the optional default coverage profile. |
| `resolve-panel` | `resolve_panel` | Resolve a panel catalog entry and validate its lock metadata. |
| `resolve-panel-lock` | `resolve_panel_lock` | Resolve and validate a panel lock entry. |
| `resolve-map` | `resolve_map` | Resolve a map catalog entry and validate its lock metadata. |
| `resolve-map-lock` | `resolve_map_lock` | Resolve and validate a map lock entry. |
| `validate-imputation-tool-compatibility` | `validate_imputation_tool_compatibility` | Validate panel/map compatibility for an imputation tool. |
| `ref-service` | `ref_service` | Access the default runtime-backed `RefService`. |

## Local Verification Commands

Run from the repository root:

```sh
CARGO_TARGET_DIR=artifacts/cargo-target cargo check -p bijux-dna-db-ref --no-default-features
CARGO_TARGET_DIR=artifacts/cargo-target cargo test -p bijux-dna-db-ref --no-default-features
```

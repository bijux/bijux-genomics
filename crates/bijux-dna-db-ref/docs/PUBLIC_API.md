# Public API

The crate root preserves a curated public surface through `src/public_api/` and
compatibility re-exports from `src/lib.rs`.

## Public Root Modules

- `public_api`

The remaining source modules are crate-private implementation namespaces:
`catalog`, `model`, `providers`, `resolution`, and `runtime_config`.

## Stable Re-Exports

`src/public_api/mod.rs` curates the stable API for downstream callers:

- catalog records: `CatalogCompatibility`, `CatalogFileEntry`,
  `MapCatalogEntry`, `MapCompatibility`, `MapLockEntry`, `PanelCatalogEntry`,
  and `PanelLockEntry`
- model records: `BuildId`, `ContigMap`, `ContigNormalizationPolicy`,
  `GeneticMapBankEntry`, `OrganellarPolicy`, `ParRegion`,
  `ReferenceBankEntry`, `ReferenceBundle`, `ReferenceProvenance`,
  `ReferenceSet`, `ResolvedSpeciesContext`, `SexChromosomeRule`,
  `SpeciesAuthorityEntry`, and `SupportedFeatures`
- provider traits and runtime provider: `RefService`, `ReferenceProvider`,
  `PanelProvider`, `MapProvider`, `RuntimeRefService`, and `ref_service`
- resolution operations such as `resolve_species_context`,
  `resolve_reference_bundle`, `resolve_panel`, `resolve_map`,
  `resolve_species_alias`, `normalize_contig_name`,
  `enforce_declared_build_and_contigs`, and
  `validate_imputation_tool_compatibility`

## Extension Rules

1. Add new stable exports through `src/public_api/mod.rs`.
2. Keep implementation modules crate-private unless a downstream caller needs a
   stable namespace.
3. Do not add a new public root module without updating `README.md`,
   `docs/ARCHITECTURE.md`, this file, and boundary tests.
4. Do not expose planner, runner, stage, environment, CLI, or API behavior
   through this crate.

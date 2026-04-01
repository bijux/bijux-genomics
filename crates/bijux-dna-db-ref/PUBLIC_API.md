# bijux-dna-db-ref Public API

The crate exposes a curated namespace in `public_api` for deterministic reference governance:

- resolution entrypoints such as `resolve_species_context`, `resolve_reference_bundle`, `resolve_panel`, and `resolve_map`
- authority and normalization helpers such as `resolve_species_alias`, `resolve_reference_bank`, `normalize_contig_name`, and `enforce_declared_build_and_contigs`
- provider contracts `RefService`, `ReferenceProvider`, `PanelProvider`, and `MapProvider`
- the default runtime-backed resolver `RuntimeRefService` and shared accessor `ref_service()`

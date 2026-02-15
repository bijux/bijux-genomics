# PUBLIC API

- `resolve_species_alias(alias, requested_build)`
- `resolve_species_authority(species)`
- `resolve_reference_bank(species, build)`
- `resolve_contig_map(species, build)`
- `resolve_genetic_map_bank(species, build, panel_id)`
- `resolve_sex_chromosome_rule(species, build)`
- `resolve_organellar_policy(species, build)`
- `resolve_default_reference_set(species, usecase)`
- `resolve_coverage_profile(species, build)`
- `resolve_species_context(species, build)`
- `resolve_reference_bundle(species, build)`
- `normalize_contig_name(bundle, contig)`
- `enforce_declared_build_and_contigs(species, declared_build, observed_contigs)`
- `reference_provenance(species, build, bundle)`

Traits:
- `ReferenceProvider`
- `PanelProvider`
- `MapProvider`

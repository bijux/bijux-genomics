# REFERENCE_GOVERNANCE

VCF stages use `bijux-dna-db-ref` for reference bundles, panel/map compatibility, contig normalization, and species/build refusal checks.

## Rules
- Use db-ref APIs (`resolve_reference_bundle`, `resolve_reference_bank`, `resolve_genetic_map_bank`) instead of direct path literals.
- Required references and maps must have lock-backed checksums.
- Runtime writes `run_artifacts/reference_manifest.json` per stage when references are involved.
